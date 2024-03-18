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

#[cfg(test)]
mod tests {
    use super::find_up;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn file_found_in_current_dir() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("testfile.txt");

        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "This is a test file.").unwrap();

        std::env::set_current_dir(&temp_dir.path()).unwrap();

        assert_eq!(find_up("testfile.txt", None), Some(file_path));
    }

    #[test]
    fn file_found_in_parent_dir() {
        let temp_dir = tempdir().unwrap();
        let child_dir = temp_dir.path().join("child");
        fs::create_dir(&child_dir).unwrap();

        let file_path = temp_dir.path().join("testfile.txt");
        File::create(&file_path).unwrap();

        std::env::set_current_dir(&child_dir).unwrap();

        assert_eq!(find_up("testfile.txt", None), Some(file_path));
    }

    #[test]
    fn file_not_found() {
        let temp_dir = tempdir().unwrap();
        let child_dir = temp_dir.path().join("child");
        fs::create_dir(&child_dir).unwrap();

        std::env::set_current_dir(&child_dir).unwrap();

        assert_eq!(find_up("nonexistent.txt", None), None);
    }

    #[test]
    fn file_not_found_due_to_max_parents() {
        let temp_dir = tempdir().unwrap();
        let level1_dir = temp_dir.path().join("level1");
        let level2_dir = level1_dir.join("level2");
        let level3_dir = level2_dir.join("level3");

        fs::create_dir_all(&level3_dir).unwrap();

        let file_path = temp_dir.path().join("testfile.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "This is a test file.").unwrap();

        std::env::set_current_dir(&level3_dir).unwrap();

        assert_eq!(find_up("testfile.txt", Some(1)), None);

        std::env::set_current_dir(Path::new("/")).unwrap();
    }
}
