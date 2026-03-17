use std::path::PathBuf;

/// Compute a relative path from `from_dir` to `to`, both absolute.
///
/// Strips the common prefix, emits one `../` per remaining component in
/// `from_dir`, then appends the remaining suffix of `to`.
pub fn relative_path(from_dir: &std::path::Path, to: &std::path::Path) -> PathBuf {
    let from_components: Vec<_> = from_dir.components().collect();
    let to_components: Vec<_> = to.components().collect();

    let common_len = from_components
        .iter()
        .zip(to_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let up_count = from_components.len() - common_len;
    let mut result = PathBuf::new();
    for _ in 0..up_count {
        result.push("..");
    }
    for component in &to_components[common_len..] {
        result.push(component);
    }
    result
}

/// Normalize a path by resolving `.` and `..` components without touching the filesystem.
/// Unlike `canonicalize`, this works on non-existent paths (e.g. broken symlink targets).
pub fn normalize_path(path: &std::path::Path) -> PathBuf {
    use std::path::Component;
    let mut components: Vec<Component> = vec![];
    for component in path.components() {
        match component {
            Component::ParentDir => {
                // Only pop if we have a non-root component to pop
                if matches!(components.last(), Some(Component::Normal(_))) {
                    components.pop();
                }
            }
            Component::CurDir => {}
            c => components.push(c),
        }
    }
    components.iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_relative_path_sibling() {
        let from = Path::new("/a/repos/canvas-lms/gems/plugins");
        let to = Path::new("/a/repos/mra");
        let rel = relative_path(from, to);
        assert_eq!(rel, PathBuf::from("../../../mra"));
    }

    #[test]
    fn test_relative_path_same_level() {
        let from = Path::new("/a/b");
        let to = Path::new("/a/c");
        let rel = relative_path(from, to);
        assert_eq!(rel, PathBuf::from("../c"));
    }
}
