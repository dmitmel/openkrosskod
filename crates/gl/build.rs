use gl_generator::{Api, DebugStructGenerator, Fallbacks, Profile, Registry, StructGenerator};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
  let dest = env::var("OUT_DIR").unwrap();
  let mut file = File::create(&Path::new(&dest).join("bindings.rs")).unwrap();

  let registry =
    Registry::new(Api::Gles2, (2, 0), Profile::Core, Fallbacks::All, ["GL_KHR_debug"]);

  if cfg!(feature = "gl_debug_all_commands") {
    registry.write_bindings(DebugStructGenerator, &mut file)
  } else {
    registry.write_bindings(StructGenerator, &mut file)
  }
  .unwrap();
}
