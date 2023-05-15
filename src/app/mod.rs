use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::str::FromStr;

static IS_PATH_CHECKED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "macos")]
pub fn data_dir() -> PathBuf {
    let path = PathBuf::from_str(
        &(std::env::var("HOME").expect("no HOME env var")
            + "/Library/Application Support/monisens/"),
    )
    .unwrap();
    check_and_create_data_dir(&path);

    path
}

#[cfg(target_os = "linux")]
pub fn data_dir() -> PathBuf {
    let path =
        PathBuf::from_str(&(std::env::var("HOME").expect("no HOME env var") + "/.monisens/"))
            .unwrap();
    check_and_create_data_dir(&path);

    path
}

#[cfg(target_os = "windows")]
pub fn data_dir() -> PathBuf {
    let mut path = std::env::current_exe().expect("failed to get current exe path");
    path.pop();

    path.push(".monisens");

    check_and_create_data_dir(&path);

    path
}

fn check_and_create_data_dir<P: AsRef<Path>>(path: P) {
    if IS_PATH_CHECKED.load(Ordering::SeqCst) {
        return;
    }

    if !path.as_ref().is_dir() {
        fs::create_dir(&path).expect(&format!("failed to create dir: '{:?}'", path.as_ref()));
    }

    IS_PATH_CHECKED.store(true, Ordering::SeqCst);
}
