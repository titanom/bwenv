use std::{env, path::PathBuf};

pub fn find_up(filename: &str, max_parents: Option<i32>) -> Option<PathBuf> {
    let current_dir = env::current_dir().ok()?;
    let mut current_path = current_dir.as_path();

    for _ in 0..max_parents.unwrap_or(10) {
        let file_path = current_path.join(filename);

        if file_path.exists() {
            return Some(file_path);
        }

        match current_path.parent() {
            Some(parent) => current_path = parent,
            None => break,
        }
    }

    None
}
