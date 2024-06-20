use std::path::{Path, PathBuf};

pub(crate) fn common_ancestor(paths: &Vec<PathBuf>) -> Option<PathBuf> {
    if paths.is_empty() {
        return None;
    }

    // Initialize with the first path
    let mut iter = paths.iter();
    let first_path = iter.next().unwrap();
    let Some(common_path) = first_path.parent() else {
        return Some(PathBuf::from("/"));
    };
    let mut common_path = common_path.to_path_buf();

    for path in iter {
        common_path = find_common_prefix(&common_path, path);
        if common_path.as_os_str().is_empty() {
            return Some(PathBuf::new()); // No common ancestor found
        }
    }

    Some(common_path)
}

fn find_common_prefix(path1: &Path, path2: &Path) -> PathBuf {
    let mut common_path = PathBuf::new();
    let mut iter1 = path1.components();
    let mut iter2 = path2.components();

    while let (Some(comp1), Some(comp2)) = (iter1.next(), iter2.next()) {
        if comp1 == comp2 {
            common_path.push(comp1);
        } else {
            break;
        }
    }

    common_path
}
