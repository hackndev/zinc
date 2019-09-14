#![feature(custom_attribute, proc_macro, rustc_macro)]

extern crate rustache;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use std::ascii::AsciiExt;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn get_platform() -> Option<String> {
  let features = env::vars().filter(|&(ref key, _)| key.starts_with("CARGO_FEATURE_MCU_"));
  match features.last() {
    Some((feature_var, _)) => Some(
      feature_var.trim_left_matches("CARGO_FEATURE_MCU_")
        .to_string().to_ascii_lowercase()),
      None => None,
  }
}

fn file_exists<P: AsRef<Path>>(file: P) -> bool {
  let file: &Path = file.as_ref();
  match fs::metadata(file) {
    Ok(_) => true,
    // Check for ENOENT (No such file or directory)
    Err(e) => e.raw_os_error() != Some(2),
  }
}

#[derive(Debug)]
enum GenerateLayoutError {
  Io(io::Error),
  Yaml(serde_yaml::Error),
  Rustache(rustache::RustacheError),
}

fn target_dir<P: AsRef<Path>>(target: P) -> PathBuf {
  Path::new("src/hal").join(target)
}

trait RustacheHash {
  fn rustache_insert<'a>(&self, hb: rustache::HashBuilder<'a>) -> rustache::HashBuilder<'a>;
}

#[derive(Serialize, Deserialize, Debug)]
struct McuLayout {
  vectors: String,
  memories: Vec<McuMemory>,
}

impl RustacheHash for McuLayout {
  fn rustache_insert<'a>(&self, hb: rustache::HashBuilder<'a>) -> rustache::HashBuilder<'a> {
    let memories = self.memories.iter().fold(rustache::VecBuilder::new(), |memories, memory| {
      memories.push(memory.rustache_insert(rustache::HashBuilder::new()))
    });
    hb
      .insert("vectors", self.vectors.clone())
      .insert("memories", memories)
  }
}

#[derive(Serialize, Deserialize, Debug)]
struct McuMemory {
  name: String,
  mode: String,
  origin: u64,
  length: u64,
}

impl RustacheHash for McuMemory {
  fn rustache_insert<'a>(&self, hb: rustache::HashBuilder<'a>) -> rustache::HashBuilder<'a> {
    hb
      .insert("name", self.name.clone())
      .insert("mode", self.mode.clone())
      .insert("origin", self.origin as i32)
      .insert("length", self.length as i32)
  }
}

fn generate_layout<P: AsRef<Path>>(target: &str, out_path: P) -> Result<(), GenerateLayoutError> {
  use rustache::Render;
  use std::io::Read;

  let layout_file = target_dir(target).join("layout.yml");
  fs::File::open(layout_file)
    .map_err(|e| GenerateLayoutError::Io(e))
    .and_then(|f| {
      let res: serde_yaml::Result<McuLayout> = serde_yaml::from_reader(f);
      res.map_err(|e| GenerateLayoutError::Yaml(e))
    })
    .and_then(|layout| {
      fs::File::open("src/hal/layout.ld.in")
        .and_then(|mut f| {
          let mut buf = String::from("");
          f.read_to_string(&mut buf)
            .map(|_| (layout, buf))
        })
        .map_err(|e| GenerateLayoutError::Io(e))
    })
    .and_then(|(layout, template)| {
      fs::OpenOptions::new().write(true).create_new(true).open(out_path)
        .map_err(|e| GenerateLayoutError::Io(e))
        .map(|f| (layout, template, f))
    })
    .and_then(|(layout, template, mut f)| {
      let layout = &layout;
      let hb = rustache::HashBuilder::new();
      let hb = layout.rustache_insert(hb);
      hb.render(&template, &mut f)
        .map_err(|e| GenerateLayoutError::Rustache(e))
    })
}

fn copy_linker_scripts<P: AsRef<Path>, Q: AsRef<Path>>(target: P, out_path: Q) -> io::Result<()> {
  let path_prefix = if env::var("CARGO_MANIFEST_DIR").unwrap().find("/examples/").is_none() {
    Path::new(".")
  } else {
    Path::new("./../..")
  };
  // Try copying the linker scripts
  let target_dir = target_dir(target);
  let out_dir: &Path = out_path.as_ref();
  try!(fs::copy(path_prefix.join("src/hal/layout_common.ld"), out_dir.join("layout_common.ld")));
  let iomem_ld = path_prefix.join(target_dir.join("iomem.ld"));
  if file_exists(iomem_ld.as_path()) {
    try!(fs::copy(iomem_ld, out_dir.join("iomem.ld")));
  } else {
    try!(fs::OpenOptions::new().create(true).write(true).open(out_dir.join("iomem.ld")));
  }
  // If the MCU has a layout.ld script, we want to override the generated one
  // with new one.
  let layout_ld = path_prefix.join(target_dir.join("layout.ld"));
  if file_exists(layout_ld.as_path()) {
      let layout_ld_out = out_dir.join("layout.ld");
      if file_exists(&layout_ld_out) {
          try!(fs::remove_file(&layout_ld_out))
      }
      try!(fs::copy(layout_ld, &layout_ld_out));
  }

  Ok(())
}

fn main() {
  let platform = match get_platform() {
    Some(p) => p,
    None => {
      return;
    },
  };

  // Get output directory for cargo for zinc crate
  let out_dir = env::var("OUT_DIR").unwrap();

  let layout_path = Path::new(&out_dir).join("layout.ld");
  if file_exists(&layout_path) {
    match fs::remove_file(&layout_path) {
      Ok(..) => {},
      Err(e) => panic!("Failed to clean layout.ld: {}", e),
    }
  }

  // Create the new layout.ld
  match generate_layout(platform.as_str(), &layout_path) {
    Ok(..) => {},
    Err(e) => panic!("Failed to create layout.ld: {:?}", e),
  }

  // Move linker scripts to cargo output dir
  match copy_linker_scripts(&platform, &out_dir) {
    Ok(_) => {},
    Err(e) => panic!("Failed to copy linker scripts: {}", e)
  }

  // Make sure that the output dir is passed to linker
  println!("cargo:rustc-link-search=native={}", out_dir);
}
