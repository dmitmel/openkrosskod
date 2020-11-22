// TODO: consider using parking_lot

use prelude_plus::*;
use std::os::unix::ffi::OsStrExt;

pub fn main() {
  const RUNS: usize = 4;

  macro_rules! bench {
    ($func:ident) => {{
      let avg = (0..RUNS)
        .map(|run| {
          let start_time = Instant::now();
          let images = $func();
          println!("{}", images.write().unwrap().len());
          let elapsed = start_time.elapsed().as_micros();
          println!("{} run #{}: {} micros", stringify!($func), run, elapsed);
          elapsed as f64
        })
        .sum::<f64>()
        / RUNS as f64;
      println!("{} avg: {} micros", stringify!($func), avg);
      println!();
      avg
    }};
  }

  let single_threaded_average = bench!(single_threaded_main);
  let multi_threaded_average = bench!(multi_threaded_main);
  println!("{:?}x boost", single_threaded_average / multi_threaded_average);
}

pub fn get_image_paths_recursively(dir: &Path) -> Vec<PathBuf> {
  let mut paths = vec![];

  let dir_entries: Vec<io::Result<(bool, PathBuf)>> = fs::read_dir(dir)
    .expect("read_dir")
    .map(|dir_entry| {
      dir_entry.map(|dir_entry| {
        let is_dir = fs::metadata(dir_entry.path()).expect("metadata").is_dir();
        (is_dir, dir_entry.path())
      })
    })
    .collect();

  for dir_entry in dir_entries {
    let (is_dir, path) = dir_entry.unwrap();
    if is_dir {
      paths.append(&mut get_image_paths_recursively(&path));
    } else if path.extension().map_or(false, |ext| ext.as_bytes() == b"png") {
      paths.push(path);
    }
  }

  paths
}

fn get_image_paths() -> Vec<PathBuf> {
  get_image_paths_recursively(Path::new("/home/dmitmel/crosscode/assets/media"))
}

fn decode_png_image<R: Read>(read: R) -> Result<Vec<u8>, png::DecodingError> {
  let buf_reader = BufReader::new(read);
  let decoder = png::Decoder::new(buf_reader);
  let (info, mut reader) = decoder.read_info().unwrap();
  let mut buf = vec![0; info.buffer_size()];
  reader.next_frame(&mut buf)?;
  Ok(buf)
}

const REPEAT_TIMES: usize = 1;

type SharedImagesMap = Arc<RwLock<HashMap<PathBuf, Arc<RwLock<Vec<u8>>>>>>;

pub fn single_threaded_main() -> SharedImagesMap {
  let image_paths = get_image_paths();
  let images: SharedImagesMap = Arc::new(RwLock::new(HashMap::with_capacity(image_paths.len())));

  {
    let mut images = images.write().unwrap();
    for _ in 0..REPEAT_TIMES {
      for path in image_paths.clone() {
        let file = File::open(&path).expect("File::open");
        let buf = decode_png_image(file).unwrap();
        images.insert(path, Arc::new(RwLock::new(buf)));
      }
    }
  }

  images
}

pub fn multi_threaded_main() -> SharedImagesMap {
  let image_paths = get_image_paths();
  let images: SharedImagesMap = Arc::new(RwLock::new(HashMap::with_capacity(image_paths.len())));
  let (decoding_requests_send, decoding_requests_recv) = mpsc::sync_channel::<PathBuf>(0);
  let decoding_requests_recv_locked = Arc::new(Mutex::new(decoding_requests_recv));

  fn spawn_thread<F, T>(name: String, f: F) -> thread::JoinHandle<T>
  where
    F: Send + 'static + FnOnce() -> T,
    T: Send + 'static,
  {
    thread::Builder::new()
      .name(name.clone())
      .spawn(f)
      .unwrap_or_else(|_| panic!("failed to spawn thread '{}'", name))
  }

  fn image_decoding_job(
    id: usize,
    images: SharedImagesMap,
    decoding_requests_recv_locked: Arc<Mutex<mpsc::Receiver<PathBuf>>>,
  ) -> thread::JoinHandle<()> {
    spawn_thread(format!("image_decoding_job({})", id), move || loop {
      let decoding_requests_recv = decoding_requests_recv_locked.lock().unwrap();
      if let Ok(image_path) = decoding_requests_recv.recv() {
        drop(decoding_requests_recv);

        let file = File::open(&image_path).expect("File::open");
        let buf = decode_png_image(file).unwrap();

        if let Ok(mut images) = images.write() {
          images.insert(image_path, Arc::new(RwLock::new(buf)));
        }
      } else {
        break;
      }
    })
  }

  let jobs_count = num_cpus::get();
  let jobs: Vec<thread::JoinHandle<()>> = (0..jobs_count)
    .map(|i| image_decoding_job(i, images.clone(), decoding_requests_recv_locked.clone()))
    .collect();

  for _ in 0..REPEAT_TIMES {
    for path in image_paths.clone() {
      decoding_requests_send.send(path).expect("decoding requests channel has been broken");
    }
  }

  drop(decoding_requests_send);

  for (i, job) in jobs.into_iter().enumerate() {
    job.join().unwrap_or_else(|_| panic!("thread of image decoding job #{} has panicked", i));
  }

  images
}
