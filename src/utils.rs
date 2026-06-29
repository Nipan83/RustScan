//! Small path helpers shared across the pipeline.

use std::path::Path;

/// Return true if the final component of `path` is a hidden name (starts with `.`).
pub fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detects_hidden_names() {
        assert!(is_hidden(Path::new(".git")));
        assert!(is_hidden(Path::new("/tmp/.env")));
        assert!(is_hidden(Path::new("dir/.hidden")));
        assert!(!is_hidden(Path::new("visible.txt")));
        assert!(!is_hidden(Path::new("/tmp/visible")));
    }
}
