use tempfile::TempDir;

use crate::funcky::{Config, FunckManager};

fn new_manager(tmp: TempDir) -> FunckManager {
    let so_directory = tmp.path().join("shared_objects");
    let build_directory = tmp.path().join("build");

    let cfg = Config {
        shared_object_directory: so_directory,
        tmp_dir: build_directory,
    };

    FunckManager::new(cfg).unwrap()
}

#[test]
pub fn test_manager_initialization() {
    let tmp = TempDir::new().unwrap();

    let so_directory = tmp.path().join("shared_objects");
    let build_directory = tmp.path().join("build");

    let cfg = Config {
        shared_object_directory: so_directory.clone(),
        tmp_dir: build_directory.clone(),
    };

    let _manager = FunckManager::new(cfg).unwrap();

    assert!(so_directory.exists());
    assert!(build_directory.exists());
}
