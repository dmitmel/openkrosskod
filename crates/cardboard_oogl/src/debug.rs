use crate::impl_prelude::*;
use prelude_plus::*;

pub(crate) fn init(gl: &RawGL) {
  if gl.DebugMessageCallback.is_loaded() {
    unsafe {
      gl.Enable(gl::DEBUG_OUTPUT);
      gl.DebugMessageCallback(Some(internal_debug_message_callback), ptr::null());
    }
  }
}

gl_enum!({
  pub enum DebugMessageSeverity {
    High = DEBUG_SEVERITY_HIGH,
    Medium = DEBUG_SEVERITY_MEDIUM,
    Low = DEBUG_SEVERITY_LOW,
    Notification = DEBUG_SEVERITY_NOTIFICATION,
  }
});

gl_enum!({
  pub enum DebugMessageSource {
    API = DEBUG_SOURCE_API,
    WindowSystem = DEBUG_SOURCE_WINDOW_SYSTEM,
    ShaderCompiler = DEBUG_SOURCE_SHADER_COMPILER,
    ThirdParty = DEBUG_SOURCE_THIRD_PARTY,
    Application = DEBUG_SOURCE_APPLICATION,
    Other = DEBUG_SOURCE_OTHER,
  }
});

gl_enum!({
  pub enum DebugMessageType {
    Error = DEBUG_TYPE_ERROR,
    DeprecatedBehavior = DEBUG_TYPE_DEPRECATED_BEHAVIOR,
    UndefinedBehavior = DEBUG_TYPE_UNDEFINED_BEHAVIOR,
    Portability = DEBUG_TYPE_PORTABILITY,
    Performance = DEBUG_TYPE_PERFORMANCE,
    Marker = DEBUG_TYPE_MARKER,
    PushGroup = DEBUG_TYPE_PUSH_GROUP,
    PopGroup = DEBUG_TYPE_POP_GROUP,
    Other = DEBUG_TYPE_OTHER,
  }
});

extern "system" fn internal_debug_message_callback(
  source: u32,
  type_: u32,
  id: u32,
  severity: u32,
  length: i32,
  message: *const c_char,
  _user_param: *mut c_void,
) {
  fn enum_to_string<T: fmt::Debug>(opt: Option<T>) -> String {
    match opt {
      Some(value) => format!("{:?}", value),
      None => "Unknown".to_owned(),
    }
  }

  let source_str = enum_to_string(DebugMessageSource::from_raw(source));
  let type_str = enum_to_string(DebugMessageType::from_raw(type_));
  let severity_str = enum_to_string(DebugMessageSeverity::from_raw(severity));
  let message_slice = unsafe { slice::from_raw_parts(message as *const u8, length as usize) };
  let message_str = String::from_utf8_lossy(message_slice);

  debug!(
    "0x{:08x} [source: {}, type: {}, severity: {}] {}",
    id, source_str, type_str, severity_str, message_str,
  );
}

pub(crate) unsafe fn unset_object_debug_label(ctx: &Context, type_id: u32, addr: u32) {
  let gl = ctx.raw_gl();
  if gl.ObjectLabel.is_loaded() {
    gl.ObjectLabel(type_id, addr, 0, ptr::null());
  }
}

pub(crate) unsafe fn set_object_debug_label(ctx: &Context, type_id: u32, addr: u32, label: &[u8]) {
  let gl = ctx.raw_gl();
  if gl.ObjectLabel.is_loaded() {
    let label_len = i32::try_from(label.len()).unwrap();
    assert!(label_len <= ctx.capabilities().max_debug_object_label_len);

    // `<[u8]>::contains` internally uses `memchr` which is faster than a
    // simple loop. Unfortunately the `memchr` implementation is not exported
    // from the std, so I can't really get an index of the offending NUL byte
    // without using less efficient methods (using just std). If the need
    // arises - either use the crate `memchr` or the unstable feature
    // `slice_internals`.
    // By the way, the behavior of ignoring characters past the NUL byte in the
    // debug label (even though length is provided explicitly) is undocumented
    // as far as I know.
    // Error message was copied from `Error` impl of `FromBytesWithNulError`.
    assert!(!label.contains(&0), "data provided contains an interior nul byte");

    gl.ObjectLabel(type_id, addr, label_len, label.as_ptr() as *const c_char);
  }
}

pub(crate) unsafe fn get_object_debug_label(ctx: &Context, type_id: u32, addr: u32) -> Vec<u8> {
  let gl = ctx.raw_gl();
  if gl.GetObjectLabel.is_loaded() {
    let buf_size =
      // The buffer will contain a NUL-terminated string, so reserve one more
      // byte for the final NUL character.
      ctx.capabilities().max_debug_object_label_len.checked_add(1).unwrap();
    let mut buf: Vec<u8> = Vec::with_capacity(buf_size as usize);
    let mut text_len: i32 = 0;

    gl.GetObjectLabel(type_id, addr, buf_size, &mut text_len, buf.as_mut_ptr() as *mut c_char);
    buf.set_len(text_len as usize);

    buf
  } else {
    vec![]
  }
}
