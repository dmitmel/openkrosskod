use crate::gl_prelude::*;
use crate::prelude::*;

gl_enum! {
  #[derive(Debug, Copy, Clone, Eq, PartialEq)]
  pub enum DebugMessageSeverity {
    High = DEBUG_SEVERITY_HIGH,
    Medium = DEBUG_SEVERITY_MEDIUM,
    Low = DEBUG_SEVERITY_LOW,
    Notification = DEBUG_SEVERITY_NOTIFICATION,
  }
}

gl_enum! {
  #[derive(Debug, Copy, Clone, Eq, PartialEq)]
  pub enum DebugMessageSource {
    API = DEBUG_SOURCE_API,
    WindowSystem = DEBUG_SOURCE_WINDOW_SYSTEM,
    ShaderCompiler = DEBUG_SOURCE_SHADER_COMPILER,
    ThirdParty = DEBUG_SOURCE_THIRD_PARTY,
    Application = DEBUG_SOURCE_APPLICATION,
    Other = DEBUG_SOURCE_OTHER,
  }
}

gl_enum! {
  #[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
}

pub extern "system" fn internal_debug_message_callback(
  source: GLenum,
  type_: GLenum,
  id: GLuint,
  severity: GLenum,
  length: GLsizei,
  message: *const GLchar,
  _user_param: *mut GLvoid,
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

  eprintln!(
    "GL debug [source = {}, type = {}, id = 0x{:x}, severity = {}]: message = {}",
    source_str, type_str, id, severity_str, message_str,
  );
}
