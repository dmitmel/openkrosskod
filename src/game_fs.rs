use prelude_plus::*;

const ASSETS_DIR_NAME: &str = "assets";

#[derive(Debug)]
pub struct GameFs {
  pub installation_dir: PathBuf,
  pub assets_dir: PathBuf,
}

impl GameFs {
  pub fn init() -> AnyResult<Self> {
    // TODO: remove the question marks and add warning logs
    for installation_dir in vec![env::current_exe()?, env::current_dir()?] {
      let assets_dir = installation_dir.join(ASSETS_DIR_NAME);
      if assets_dir.exists() {
        return Ok(Self { installation_dir, assets_dir });
      }
    }
    // bail!("Failed to find the installation directory")
    Ok(Self { installation_dir: PathBuf::from("."), assets_dir: PathBuf::from("./assets") })
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
    let mut bytes = Vec::with_capacity(file.metadata().map_or(0, |m| m.len() as usize + 1));
    file
      .read_to_end(&mut bytes)
      .with_context(|| format!("Failed to read file '{}'", relative_path.display()))?;
    Ok(bytes)
  }
}
