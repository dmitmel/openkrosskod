pub fn result_map_both<T, U, F: FnOnce(T) -> U>(result: Result<T, T>, op: F) -> Result<U, U> {
  match result {
    Ok(t) => Ok(op(t)),
    Err(e) => Err(op(e)),
  }
}
