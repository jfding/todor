pub mod cli;
pub mod taskbox;

use std::path::PathBuf;
use std::fs;
use dirs;

const DATA_BASE : &str = ".local/share/todor";

pub fn get_inbox_file(dir: Option<String>, inbox: Option<String>) -> PathBuf {
    let base_path = dir.map(PathBuf::from).unwrap_or_else(|| {
        dirs::home_dir()
            .expect("cannot get home directory")
            .join(DATA_BASE)
    });
    fs::create_dir_all(&base_path).expect("Failed to create base directory");

    return base_path.join(inbox.unwrap_or("TODO".to_string())).with_extension("md");
}

