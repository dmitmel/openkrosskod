use prelude_plus::*;

const ASSETS_DIR_NAME: &str = "assets";

#[derive(Debug)]
pub struct GameFs {
  pub installation_dir: PathBuf,
  pub assets_dir: PathBuf,
}

impl GameFs {
  pub fn init() -> AnyResult<Self> {
    let exe_path: PathBuf = env::current_exe()
      .context("Failed to get the executable path")?
      .canonicalize()
      .context("Failed to canonicalize the executable path")?;
    info!("Executable path:  '{}'", exe_path.display());

    let exe_dir: &Path =
      exe_path.parent().ok_or_else(|| format_err!("The executable path must have a parent"))?;
    for installation_dir in exe_dir.ancestors() {
      let assets_dir = installation_dir.join(ASSETS_DIR_NAME);
      if !assets_dir.is_dir() {
        continue;
      }

      info!("Installation dir: '{}'", installation_dir.display());

      return Ok(Self { installation_dir: installation_dir.to_owned(), assets_dir });
    }

    bail!("Failed to find the installation directory")
  }

  pub fn open_file<P: AsRef<Path>>(&self, relative_path: P) -> AnyResult<File> {
    self._open_file(relative_path.as_ref())
  }

  fn _open_file(&self, relative_path: &Path) -> AnyResult<File> {
    File::open(self.assets_dir.join(relative_path))
      .with_context(|| format!("Failed to open file '{}'", relative_path.display()))
  }

  pub fn read_binary_file<P: AsRef<Path>>(&self, relative_path: P) -> AnyResult<Vec<u8>> {
    self._read_binary_file(relative_path.as_ref())
  }

  fn _read_binary_file(&self, relative_path: &Path) -> AnyResult<Vec<u8>> {
    let mut file = self._open_file(relative_path)?;
    let mut bytes = Vec::with_capacity(
      // see the private function initial_buffer_size in std::fs
      file.metadata().map_or(0, |m| m.len() as usize + 1),
    );
    file
      .read_to_end(&mut bytes)
      .with_context(|| format!("Failed to read file '{}'", relative_path.display()))?;
    Ok(bytes)
  }
}
