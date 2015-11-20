use std::ascii::AsciiExt;
use std::env;
use std::fs;
use std::io;
use std::path::Path;

fn get_platform() -> Option<String> {
  let features = env::vars().filter(|&(ref key, _)| key.starts_with("CARGO_FEATURE_MCU_"));
  match features.last() {
    Some((feature_var, _)) => Some(
      feature_var.trim_left_matches("CARGO_FEATURE_MCU_")
        .to_string().to_ascii_lowercase()),
      None => None,
  }
}

fn file_exists(file: &Path) -> bool {
    match fs::metadata(file) {
        Ok(_) => true,
        // Check for ENOENT (No such file or directory)
        Err(e) => e.raw_os_error() != Some(2),
    }
}

fn copy_linker_scripts<P: AsRef<Path>, Q: AsRef<Path>>(target: P, out_path: Q) -> io::Result<()> {
  let path_prefix = if env::var("CARGO_MANIFEST_DIR").unwrap().find("/examples/").is_none() {
    Path::new(".")
  } else {
    Path::new("./../..")
  };
  // Try copying the linker scripts
  let target_dir = Path::new("src/hal").join(target);
  let out_dir: &Path = out_path.as_ref();
  try!(fs::copy(path_prefix.join("src/hal/layout_common.ld"), out_dir.join("layout_common.ld")));
  let iomem_ld = path_prefix.join(target_dir.join("iomem.ld"));
  if file_exists(iomem_ld.as_path()) {
    try!(fs::copy(iomem_ld, out_dir.join("iomem.ld")));
  }
  try!(fs::copy(path_prefix.join(target_dir.join("layout.ld")), out_dir.join("layout.ld")));

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

  // Move linker scripts to cargo output dir
  match copy_linker_scripts(&platform, &out_dir) {
    Ok(_) => {},
    Err(e) => panic!("Failed to copy linker scripts: {}", e)
  }

  // Make sure that the output dir is passed to linker
  println!("cargo:rustc-link-search=native={}", out_dir);
}
