use prelude_plus::*;

#[cfg(unix)]
pub fn normalize_separators(path: &Path) -> Cow<Path> { Cow::Borrowed(path) }

#[cfg(windows)]
pub fn normalize_separators(path: &Path) -> Cow<Path> {
  use std::os::windows::ffi::{OsStrExt, OsStringExt};

  let path_chars = path.as_os_str().encode_wide();
  let mut path_chars_vec: Option<Vec<u16>> = None;

  for (i, b) in path_chars.clone().enumerate() {
    if b == b'/' as u16 {
      let vec = path_chars_vec.get_or_insert_with(|| path_chars.clone().collect());
      vec[i] = b'\\' as u16;
    }
  }

  match path_chars_vec {
    Some(path_chars_vec) => Cow::Owned(PathBuf::from(OsString::from_wide(&path_chars_vec))),
    None => Cow::Borrowed(path),
  }
}
