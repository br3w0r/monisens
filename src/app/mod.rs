use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

static IS_PATH_CHECKED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "macos")]
pub fn data_dir() -> String {
    let path =
        std::env::var("HOME").expect("no HOME env var") + "/Library/Application Support/monisens/";
    check_and_create_data_dir(&path);

    path
}

#[cfg(target_os = "linux")]
pub fn data_dir() -> String {
    let path = std::env::var("HOME").expect("no HOME env var") + "/.monisens/";
    check_and_create_data_dir();

    path
}

#[cfg(target_os = "windows")]
pub fn data_dir() -> String {
    let path = std::env::var("APP_DATA").expect("no APP_DATA env var") + "/.monisens/";
    check_and_create_data_dir();

    path
}

fn check_and_create_data_dir(path: &str) {
    if IS_PATH_CHECKED.load(Ordering::SeqCst) {
        return;
    }

    let p = Path::new(path);
    if !p.is_dir() {
        fs::create_dir(p).unwrap();
    }

    IS_PATH_CHECKED.store(true, Ordering::SeqCst);
}
