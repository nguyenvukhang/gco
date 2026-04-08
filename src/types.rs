use std::path::{Path, PathBuf};
use std::{env, fs};

pub struct Test {
    // Root directory for the test files.
    root_dir: PathBuf,
    // Unique id for anything.
    id: u32,
}

impl Test {
    pub fn new(suffix: &'static str) -> Self {
        let temp_dir = env::temp_dir().join(suffix);
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        Self { root_dir: temp_dir, id: 0 }
    }

    pub fn as_path(&self) -> &Path {
        self.root_dir.as_path()
    }

    // Gets a unique id.
    pub fn id(&mut self) -> u32 {
        self.id += 1;
        self.id
    }
}

impl Drop for Test {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root_dir);
    }
}
