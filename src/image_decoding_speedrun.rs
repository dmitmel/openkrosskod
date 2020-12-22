// TODO: handle errors

#[cfg(not(feature = "simd-json"))]
use serde_json as json;
#[cfg(not(feature = "simd-json"))]
use serde_json::Value as JsonStruct;

#[cfg(feature = "simd-json")]
use simd_json as json;
#[cfg(feature = "simd-json")]
use simd_json::OwnedValue as JsonStruct;

use prelude_plus::*;

pub fn main() {
  const RUNS: usize = 4;

  macro_rules! bench {
    ($func:ident) => {{
      let avg = (0..RUNS)
        .map(|run| {
          let start_time = Instant::now();
          drop(std::hint::black_box($func()));
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

#[derive(Debug)]
pub enum AssetFileType {
  Png,
  Json,
}

#[derive(Debug)]
pub enum Asset {
  Image { buffer: Vec<u8> },
  Json { data: JsonStruct },
}

pub fn get_assets_paths_recursively(dir: &Path) -> Vec<(PathBuf, AssetFileType)> {
  let mut paths = vec![];
  let png_extension = OsStr::new("png");
  let json_extension = OsStr::new("json");

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
      paths.append(&mut get_assets_paths_recursively(&path));
    } else {
      let file_type = path.extension().and_then(|ext| {
        Some(if ext == png_extension {
          AssetFileType::Png
        } else if ext == json_extension {
          AssetFileType::Json
        } else {
          return None;
        })
      });

      if let Some(file_type) = file_type {
        paths.push((path, file_type));
      }
    }
  }

  paths
}

fn get_asset_paths() -> Vec<(PathBuf, AssetFileType)> {
  let crosscode_root = Path::new("/home/dmitmel/all-crosscode-versions");
  // return get_assets_paths_recursively(crosscode_root);

  let mut assets = Vec::with_capacity(LOADED_IMAGE_ASSETS.len() + LOADED_JSON_ASSETS.len());

  for path in LOADED_IMAGE_ASSETS {
    assets.push((crosscode_root.join(Path::new(path)), AssetFileType::Png));
  }
  for path in LOADED_JSON_ASSETS {
    assets.push((crosscode_root.join(Path::new(path)), AssetFileType::Json));
  }

  assets
}

fn load_asset(path: &Path, file_type: AssetFileType) -> Asset {
  // println!("{}", path.display());

  let file = File::open(path).unwrap_or_else(|err| panic!("'{}': {:?}", path.display(), err));
  let buf_reader = BufReader::new(file);
  let asset = match file_type {
    AssetFileType::Png => decode_png_image(buf_reader).unwrap(),
    AssetFileType::Json => parse_json_asset(buf_reader).unwrap(),
  };

  fn decode_png_image(buf_reader: BufReader<File>) -> Result<Asset, png::DecodingError> {
    let decoder = png::Decoder::new(buf_reader);
    let (info, mut reader) = decoder.read_info().unwrap();
    let mut buffer = vec![0; info.buffer_size()];
    reader.next_frame(&mut buffer)?;
    Ok(Asset::Image { buffer })
  }

  fn parse_json_asset(read: BufReader<File>) -> json::Result<Asset> {
    Ok(Asset::Json { data: json::from_reader(read)? })
  }

  asset
}

type AssetsMap = HashMap<PathBuf, Asset>;
type SharedAssetsMap = Arc<RwLock<AssetsMap>>;

pub fn single_threaded_main() -> AssetsMap {
  let asset_paths = get_asset_paths();
  let mut assets: AssetsMap = HashMap::with_capacity(asset_paths.len());

  for (path, file_type) in asset_paths {
    let asset = load_asset(&path, file_type);
    assets.insert(path, asset);
  }

  assets
}

pub fn multi_threaded_main() -> AssetsMap {
  let asset_paths = get_asset_paths();
  let assets: SharedAssetsMap = Arc::new(RwLock::new(HashMap::with_capacity(asset_paths.len())));
  let (decoding_requests_send, decoding_requests_recv) =
    mpsc::sync_channel::<(PathBuf, AssetFileType)>(0);
  let decoding_requests_recv_locked = Arc::new(Mutex::new(decoding_requests_recv));

  fn spawn_thread<F, T>(name: String, f: F) -> thread::JoinHandle<T>
  where
    F: Send + 'static + FnOnce() -> T,
    T: Send + 'static,
  {
    thread::Builder::new()
      .name(name.clone())
      .spawn(f)
      .unwrap_or_else(|err| panic!("failed to spawn thread '{}': {:?}", name, err))
  }

  fn asset_loading_job(
    id: usize,
    assets: SharedAssetsMap,
    decoding_requests_recv_locked: Arc<Mutex<mpsc::Receiver<(PathBuf, AssetFileType)>>>,
  ) -> thread::JoinHandle<()> {
    spawn_thread(format!("asset_loading_job({})", id), move || {
      while let Some((path, file_type)) =
        decoding_requests_recv_locked.lock().ok().and_then(|recv| recv.recv().ok())
      {
        let asset = load_asset(&path, file_type);
        if let Ok(mut images) = assets.write() {
          images.insert(path, asset);
        } else {
          break;
        }
      }
    })
  }

  let jobs_count = num_cpus::get();
  let jobs: Vec<_> = (0..jobs_count)
    .map(|i| asset_loading_job(i, Arc::clone(&assets), Arc::clone(&decoding_requests_recv_locked)))
    .collect();

  for path in asset_paths {
    decoding_requests_send.send(path).expect("decoding requests channel has been broken");
  }

  drop(decoding_requests_send);

  for (i, job) in jobs.into_iter().enumerate() {
    job
      .join()
      .unwrap_or_else(|err| panic!("thread of image decoding job #{} has panicked: {:?}", i, err));
  }

  RwLock::into_inner(Arc::try_unwrap(assets).unwrap()).unwrap()
}

const LOADED_JSON_ASSETS: &[&str] = &[
  "assets/data/global-settings.json",
  "assets/data/effects/map/door.json",
  "assets/data/parallax/title.json",
  "assets/data/effects/puzzle.json",
  "assets/data/effects/stepFx.json",
  "assets/data/effects/default-hit.json",
  "assets/data/effects/guard.json",
  "assets/data/effects/combatant.json",
  "assets/data/effects/throw.json",
  "assets/data/effects/combat/mode.json",
  "assets/data/effects/drops.json",
  "assets/data/effects/ball-assault.json",
  "assets/data/effects/special-neutral.json",
  "assets/data/effects/marble.json",
  "assets/data/effects/combat/quadroguard.json",
  "assets/data/effects/sweeps.json",
  "assets/data/effects/scene/upgrade.json",
  "assets/data/effects/npc.json",
  "assets/data/effects/puzzle/bomb.json",
  "assets/data/effects/scene/water.json",
  "assets/data/effects/ball-special.json",
  "assets/data/effects/puzzle/key.json",
  "assets/data/effects/puzzle/compressor.json",
  "assets/data/effects/puzzle/water-bubble.json",
  "assets/data/effects/puzzle/ball-changer.json",
  "assets/data/effects/puzzle/lorry.json",
  "assets/data/effects/teleport.json",
  "assets/data/effects/puzzle/shield.json",
  "assets/data/effects/puzzle/magnet.json",
  "assets/data/effects/puzzle/thermo-pole.json",
  "assets/data/effects/puzzle/sliding-block.json",
  "assets/data/effects/puzzle/destructible.json",
  "assets/data/effects/map/chest.json",
  "assets/data/effects/puzzle/quicksand.json",
  "assets/data/effects/puzzle/tesla.json",
  "assets/data/effects/puzzle/wave-teleport.json",
  "assets/data/save-presets/2-continue-story.json",
  "assets/data/save-presets/3-autumn-rise.json",
  "assets/data/save-presets/1-rhombus-dng-start.json",
  "assets/data/save-presets/0-before-boss.json",
  "assets/data/save-presets/4-apollo-duel.json",
  "assets/data/save-presets/6-before-maroon.json",
  "assets/data/effects/dust.json",
  "assets/data/effects/speedlines.json",
  "assets/data/lang/sc/gimmick.en_US.json",
  "assets/data/lang/sc/map-content.en_US.json",
  "assets/data/save-presets/5-before-bergen.json",
  "assets/data/changelog.json",
  "assets/data/skilltree.json",
  "assets/data/effects/combat.json",
  "assets/data/effects/combat/triblader.json",
  "assets/data/effects/puzzle/ferro.json",
  "assets/data/save-presets/7-fajro-temple.json",
  "assets/data/save-presets/8-autumns-fall.json",
  "assets/data/effects/arena.json",
  "assets/data/lang/sc/gui.en_US.json",
  "assets/data/tile-infos.json",
  "assets/data/terrain.json",
  "assets/data/item-database.json",
  "assets/data/players/lea.json",
  "assets/data/effects/ball.json",
  "assets/data/effects/ball-heat.json",
  "assets/data/effects/ball-cold.json",
  "assets/data/effects/ball-shock.json",
  "assets/data/effects/ball-wave.json",
  "assets/data/effects/specials/icicles.json",
  "assets/data/characters/main/lea.json",
  "assets/data/animations/player.json",
  "assets/data/effects/trail.json",
  "assets/data/effects/specials/neutral.json",
  "assets/data/effects/specials/shock.json",
  "assets/data/effects/specials/heat.json",
  "assets/data/effects/specials/cold.json",
  "assets/data/effects/specials/wave.json",
  "assets/data/database.json",
  "assets/data/areas/arid-dng-1.json",
  "assets/data/areas/arid-dng-2.json",
  "assets/data/areas/cargo-ship.json",
  "assets/data/areas/fallback.json",
  "assets/data/areas/hideout.json",
  "assets/data/areas/meta.json",
  "assets/data/areas/beach.json",
  "assets/data/areas/bergen.json",
  "assets/data/areas/evo-village.json",
  "assets/data/areas/heat-dng.json",
  "assets/data/areas/heat-village.json",
  "assets/data/areas/jungle-city.json",
  "assets/data/areas/rhombus-dng.json",
  "assets/data/areas/shock-dng.json",
  "assets/data/areas/tree-dng.json",
  "assets/data/areas/wave-dng.json",
  "assets/data/areas/autumn-area.json",
  "assets/data/areas/cold-dng.json",
  "assets/data/areas/rookie-harbor.json",
  "assets/data/areas/arid.json",
  "assets/data/areas/bergen-trails.json",
  "assets/data/areas/autumn-fall.json",
  "assets/data/areas/final-dng.json",
  "assets/data/areas/heat-area.json",
  "assets/data/areas/jungle.json",
  "assets/data/areas/forest.json",
  "assets/data/areas/rhombus-sqr.json",
  "assets/data/players/shizuka.json",
  "assets/data/players/shizuka0.json",
  "assets/data/players/sergey.json",
  "assets/data/players/schneider.json",
  "assets/data/players/schneider2.json",
  "assets/data/players/hlin.json",
  "assets/data/players/grumpy.json",
  "assets/data/players/buggy.json",
  "assets/data/players/triblader1.json",
  "assets/data/players/luke.json",
  "assets/data/characters/main/sergey.json",
  "assets/data/characters/main/emilie.json",
  "assets/data/characters/main/glasses.json",
  "assets/data/characters/antagonists/fancyguy.json",
  "assets/data/characters/antagonists/sidekick.json",
  "assets/data/characters/misc/radical-fish.json",
  "assets/data/characters/main/schneider.json",
  "assets/data/animations/player-poses.json",
  "assets/data/characters/guests/sao.json",
  "assets/data/effects/enemies/sao.json",
  "assets/data/characters/main/schneider2.json",
  "assets/data/characters/main/shizuka.json",
  "assets/data/characters/main/luke.json",
  "assets/data/characters/main/guild-leader.json",
  "assets/data/characters/main/grumpy.json",
  "assets/data/characters/main/buggy.json",
  "assets/data/characters/greenies/female-researcher.json",
  "assets/data/animations/shizuka.json",
  "assets/data/animations/enemies/seahorse.json",
  "assets/data/animations/npc/schneider.json",
  "assets/data/animations/npc/guild-leader.json",
  "assets/data/effects/combat/pentafist.json",
  "assets/data/animations/npc/grumpy.json",
  "assets/data/animations/npc/buggy.json",
  "assets/data/characters/party-tmp/triblader-1.json",
  "assets/data/animations/npc/triblader-1.json",
  "assets/data/animations/npc/luke.json",
  "assets/data/players/glasses.json",
  "assets/data/players/joern.json",
  "assets/data/players/emilie.json",
  "assets/data/effects/combat/hexacast.json",
  "assets/data/animations/npc/glasses.json",
  "assets/data/animations/npc/sidekick.json",
  "assets/data/animations/npc/emilie.json",
  "assets/data/players/apollo.json",
  "assets/data/animations/npc/fancyguy.json",
  "assets/data/effects/enemies/arid.json",
  "assets/data/effects/enemies/jungle.json",
];

const LOADED_IMAGE_ASSETS: &[&str] = &[
  "assets/media/map/cloud.png",
  "assets/media/map/rain-drop.png",
  "assets/media/gui/env-white.png",
  "assets/media/map/rain.png",
  "assets/media/gui/env-red.png",
  "assets/media/gui/env-black.png",
  "assets/media/font/icons-small.png",
  "assets/media/font/icons.png",
  "assets/media/font/icons-keyboard.png",
  "assets/media/font/icons-items.png",
  "assets/media/font/icons-buff.png",
  "assets/media/font/icons-buff-large.png",
  "assets/media/font/languages.png",
  "assets/media/font/colors/hall-fetica-bold-red.png",
  "assets/media/font/colors/hall-fetica-bold-green.png",
  "assets/media/font/colors/hall-fetica-bold-purple.png",
  "assets/media/font/colors/hall-fetica-bold-grey.png",
  "assets/media/font/colors/hall-fetica-small-orange.png",
  "assets/media/font/colors/tiny-orange.png",
  "assets/media/font/colors/hall-fetica-small-purple.png",
  "assets/media/font/colors/hall-fetica-small-grey.png",
  "assets/media/font/colors/tiny-grey.png",
  "assets/media/font/colors/hall-fetica-small-red.png",
  "assets/media/font/colors/tiny-red.png",
  "assets/media/font/colors/hall-fetica-small-green.png",
  "assets/media/font/hall-fetica-bold.png",
  "assets/media/font/hall-fetica-small.png",
  "assets/media/font/tiny.png",
  "assets/media/gui/buttons.png",
  "assets/media/gui/message.png",
  "assets/media/gui/basic.png",
  "assets/media/font/colors/tiny-green.png",
  "assets/media/gui/status-gui.png",
  "assets/media/gui/severed-heads.png",
  "assets/media/gui/overload-overlay.png",
  "assets/media/gui/loading.png",
  "assets/media/gui/rfg-fish.png",
  "assets/media/gui/rfg-text.png",
  "assets/media/gui/tech-intro-bg.png",
  "assets/media/gui/html5-logo.png",
  "assets/media/gui/impact-logo.png",
  "assets/media/gui/title-logo.png",
  "assets/media/gui/new-game.png",
  "assets/media/gui/title-bg.png",
  "assets/media/gui/scanlines.png",
  "assets/media/gui/pause_word.png",
  "assets/media/gui/indiegogo.png",
  "assets/media/gui/equip-fx.png",
  "assets/media/gui/circuit.png",
  "assets/media/gui/circuit-icons.png",
  "assets/media/gui/trade-types.png",
  "assets/media/gui/area-icons.png",
  "assets/media/gui/world-map-extra.png",
  "assets/media/gui/world-map.png",
  "assets/media/gui/arena-gui.png",
  "assets/media/env/particle.png",
  "assets/media/gui/map-ar.png",
  "assets/media/entity/enemy/combatant-marble.png",
  "assets/media/entity/map-gui/hit-numbers.png",
  "assets/media/entity/player/item-hold.png",
  "assets/media/entity/enemy/drops.png",
  "assets/media/entity/enemy/item-drops.png",
  "assets/media/gui/pvp.png",
  "assets/media/gui/map-icon.png",
  "assets/media/entity/map-gui/crosshair.png",
  "assets/media/entity/objects/block.png",
  "assets/media/entity/shadow.png",
  "assets/media/map/lightmap.png",
  "assets/media/entity/effects/particles1.png",
  "assets/media/entity/effects/rhombus.png",
  "assets/media/parallax/title/sky.png",
  "assets/media/parallax/title/planet.png",
  "assets/media/parallax/title/clouds-2.png",
  "assets/media/parallax/title/clouds-1.png",
  "assets/media/parallax/title/ground.png",
  "assets/media/gui/title-logo-new.png",
  "assets/media/parallax/title/railings.png",
  "assets/media/parallax/logo/glow.png",
  "assets/media/parallax/title/lea.png",
  "assets/media/gui/menu.png",
  "assets/media/entity/effects/lighter-particle.png",
  "assets/media/entity/effects/dust.png",
  "assets/media/gui/rhombus-map.png",
  "assets/media/entity/effects/hit1.png",
  "assets/media/entity/effects/ball.png",
  "assets/media/entity/effects/hit2.png",
  "assets/media/entity/effects/guard.png",
  "assets/media/entity/effects/spread1.png",
  "assets/media/entity/effects/explosion.png",
  "assets/media/entity/effects/spread2.png",
  "assets/media/entity/effects/heat.png",
  "assets/media/entity/effects/shock.png",
  "assets/media/entity/effects/element-change.png",
  "assets/media/entity/effects/sweep2.png",
  "assets/media/entity/effects/sweep.png",
  "assets/media/map/fog2.png",
  "assets/media/entity/effects/bomb-explo.png",
  "assets/media/entity/effects/ball-special.png",
  "assets/media/entity/effects/icicles.png",
  "assets/media/entity/effects/lighter-particle-big.png",
  "assets/media/entity/enemy/turret-arid-projectile.png",
  "assets/media/entity/effects/explosion-round.png",
  "assets/media/entity/effects/explosion-round-l.png",
  "assets/media/gui/chapters.png",
  "assets/media/gui/feat-icons.png",
  "assets/media/entity/effects/quadroguard.png",
  "assets/media/entity/effects/special-charge.png",
  "assets/media/entity/effects/triblader.png",
  "assets/media/entity/effects/wave.png",
  "assets/media/entity/effects/raid-particles.png",
  "assets/media/entity/effects/final-laser.png",
  "assets/media/entity/enemy/heat-projectile.png",
  "assets/media/entity/effects/cold.png",
  "assets/media/face/lea.png",
  "assets/media/entity/player/move.png",
  "assets/media/entity/player/throw.png",
  "assets/media/entity/effects/neutral.png",
  "assets/media/entity/effects/clock-block.png",
  "assets/media/entity/enemy/jungle-projectiles.png",
  "assets/media/face/lea-hand.png",
  "assets/media/face/lea-special.png",
  "assets/media/face/programmer.png",
  "assets/media/face/fancyguy.png",
  "assets/media/face/sidekick.png",
  "assets/media/face/other.png",
  "assets/media/face/emilie.png",
  "assets/media/face/glasses.png",
  "assets/media/face/guest/sao.png",
  "assets/media/entity/npc/guest/sao.png",
  "assets/media/entity/effects/designer-magic.png",
  "assets/media/face/shizuka.png",
  "assets/media/face/luke.png",
  "assets/media/face/schneider.png",
  "assets/media/entity/player/poses.png",
  "assets/media/face/guild-leader.png",
  "assets/media/face/grumpy.png",
  "assets/media/face/buggy.png",
  "assets/media/face/tulips.png",
  "assets/media/entity/player/throw-shizuka.png",
  "assets/media/entity/player/shizuka-special.png",
  "assets/media/entity/enemy/seahorse.png",
  "assets/media/entity/npc/guild-leader.png",
  "assets/media/entity/effects/pentafist-punch.png",
  "assets/media/entity/npc/grumpy.png",
  "assets/media/entity/npc/buggy.png",
  "assets/media/face/avatars-new.png",
  "assets/media/entity/npc/triblader-battle.png",
  "assets/media/entity/npc/luke.png",
  "assets/media/entity/player/move-shizuka.png",
  "assets/media/entity/npc/schneider.png",
  "assets/media/entity/npc/triblader.png",
  "assets/media/entity/npc/runners.png",
  "assets/media/entity/npc/sidekick.png",
  "assets/media/entity/npc/emilie-attack.png",
  "assets/media/entity/npc/glasses.png",
  "assets/media/entity/effects/arid-fx.png",
  "assets/media/entity/npc/emilie.png",
  "assets/media/entity/npc/fancyguy.png",
];
