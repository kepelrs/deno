// Copyright 2018-2020 the Deno authors. All rights reserved. MIT license.
use super::dispatch_json::{Deserialize, JsonOp, Value};
use crate::fmt_errors::JSError;
use crate::op_error::OpError;
use crate::ops::json_op;
use crate::source_maps::get_orig_position;
use crate::source_maps::CachedMaps;
use crate::state::State;
use deno_core::*;
use std::collections::HashMap;

pub fn init(i: &mut Isolate, s: &State) {
  i.register_op(
    "apply_source_map",
    s.core_op(json_op(s.stateful_op(op_apply_source_map))),
  );
  i.register_op(
    "format_error",
    s.core_op(json_op(s.stateful_op(op_format_error))),
  );
}

#[derive(Deserialize)]
struct FormatErrorArgs {
  error: String,
}

fn op_format_error(
  state: &State,
  args: Value,
  _zero_copy: Option<ZeroCopyBuf>,
) -> Result<JsonOp, OpError> {
  let args: FormatErrorArgs = serde_json::from_value(args)?;
  let error =
    JSError::from_json(&args.error, &state.borrow().global_state.ts_compiler);

  Ok(JsonOp::Sync(json!({
    "error": error.to_string(),
  })))
}

#[derive(Deserialize)]
struct ApplySourceMap {
  filename: String,
  line: i32,
  column: i32,
}

fn op_apply_source_map(
  state: &State,
  args: Value,
  _zero_copy: Option<ZeroCopyBuf>,
) -> Result<JsonOp, OpError> {
  let args: ApplySourceMap = serde_json::from_value(args)?;

  let mut mappings_map: CachedMaps = HashMap::new();
  let (orig_filename, orig_line, orig_column) = get_orig_position(
    args.filename,
    args.line.into(),
    args.column.into(),
    &mut mappings_map,
    &state.borrow().global_state.ts_compiler,
  );

  Ok(JsonOp::Sync(json!({
    "filename": orig_filename,
    "line": orig_line as u32,
    "column": orig_column as u32,
  })))
}
