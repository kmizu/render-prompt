use crate::error::RenderError;
use lazy_static::lazy_static;
use path_clean::PathClean;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

lazy_static! {
    // Match {{> path/to/file }}
    static ref INCLUDE_PATTERN: Regex = Regex::new(r"\{\{>\s*([^}]+?)\s*\}\}").unwrap();
}

pub struct IncludeResolver {
    root_dir: PathBuf,
    max_depth: usize,
}

impl IncludeResolver {
    pub fn new<P: AsRef<Path>>(root_dir: P, max_depth: usize) -> Self {
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
            max_depth,
        }
    }

    /// Resolve all includes in the content recursively
    pub fn resolve(
        &self,
        content: &str,
        current_file: &Path,
        visited: &mut HashSet<PathBuf>,
        depth: usize,
    ) -> Result<String, RenderError> {
        // Check depth limit
        if depth > self.max_depth {
            return Err(RenderError::IncludeDepthExceeded {
                max_depth: self.max_depth,
            });
        }

        let mut result = String::with_capacity(content.len());
        let mut last_end = 0;

        for cap in INCLUDE_PATTERN.captures_iter(content) {
            let full_match = cap.get(0).unwrap();
            let start = full_match.start();
            let end = full_match.end();

            // Add text before this match
            result.push_str(&content[last_end..start]);

            // Extract the include path
            let include_path = cap.get(1).unwrap().as_str().trim();

            // Resolve the path
            let resolved_path = self.resolve_path(current_file, include_path)?;

            // Check for circular include
            if visited.contains(&resolved_path) {
                return Err(RenderError::CircularInclude {
                    path: resolved_path.display().to_string(),
                });
            }

            // Check path traversal (ensure it's within root)
            if !self.is_within_root(&resolved_path)? {
                return Err(RenderError::PathTraversal {
                    path: include_path.to_string(),
                });
            }

            // Read the included file
            let included_content =
                fs::read_to_string(&resolved_path).map_err(|e| RenderError::IncludeFileRead {
                    path: resolved_path.display().to_string(),
                    source: e,
                })?;

            // Mark as visited
            visited.insert(resolved_path.clone());

            // Recursively resolve includes in the included content
            let expanded = self.resolve(&included_content, &resolved_path, visited, depth + 1)?;

            // Add expanded content
            result.push_str(&expanded);

            // Unmark (allow including the same file from different branches)
            visited.remove(&resolved_path);

            last_end = end;
        }

        // Add remaining text
        result.push_str(&content[last_end..]);

        Ok(result)
    }

    /// Resolve a relative include path to an absolute path
    fn resolve_path(&self, current_file: &Path, relative_path: &str) -> Result<PathBuf, RenderError> {
        // Get the directory of the current file
        let current_dir = current_file
            .parent()
            .unwrap_or_else(|| Path::new("."));

        // Join with the relative path
        let joined = current_dir.join(relative_path);

        // Clean the path (resolve . and ..)
        let cleaned = joined.clean();

        Ok(cleaned)
    }

    /// Check if a path is within the root directory
    fn is_within_root(&self, path: &Path) -> Result<bool, RenderError> {
        // Canonicalize both paths to resolve symlinks and get absolute paths
        let canonical_path = path
            .canonicalize()
            .map_err(|e| RenderError::IncludeFileRead {
                path: path.display().to_string(),
                source: e,
            })?;

        let canonical_root = self
            .root_dir
            .canonicalize()
            .map_err(|e| RenderError::IncludeFileRead {
                path: self.root_dir.display().to_string(),
                source: e,
            })?;

        Ok(canonical_path.starts_with(&canonical_root))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_simple_include() {
        let dir = tempdir().unwrap();

        // Create a main file and an include file
        let include_file = dir.path().join("included.txt");
        fs::write(&include_file, "Hello from include!").unwrap();

        let main_file = dir.path().join("main.txt");
        fs::write(&main_file, "Start\n{{> included.txt }}\nEnd").unwrap();

        let resolver = IncludeResolver::new(dir.path(), 20);
        let content = fs::read_to_string(&main_file).unwrap();
        let mut visited = HashSet::new();

        let result = resolver.resolve(&content, &main_file, &mut visited, 0).unwrap();
        assert_eq!(result, "Start\nHello from include!\nEnd");
    }

    #[test]
    fn test_nested_include() {
        let dir = tempdir().unwrap();

        // Create three files: main -> a -> b
        let file_b = dir.path().join("b.txt");
        fs::write(&file_b, "Content B").unwrap();

        let file_a = dir.path().join("a.txt");
        fs::write(&file_a, "Content A\n{{> b.txt }}").unwrap();

        let main_file = dir.path().join("main.txt");
        fs::write(&main_file, "Main\n{{> a.txt }}").unwrap();

        let resolver = IncludeResolver::new(dir.path(), 20);
        let content = fs::read_to_string(&main_file).unwrap();
        let mut visited = HashSet::new();

        let result = resolver.resolve(&content, &main_file, &mut visited, 0).unwrap();
        assert_eq!(result, "Main\nContent A\nContent B");
    }

    #[test]
    fn test_circular_include() {
        let dir = tempdir().unwrap();

        // Create circular reference: a -> b -> a
        let file_a = dir.path().join("a.txt");
        let file_b = dir.path().join("b.txt");

        fs::write(&file_a, "A {{> b.txt }}").unwrap();
        fs::write(&file_b, "B {{> a.txt }}").unwrap();

        let resolver = IncludeResolver::new(dir.path(), 20);
        let content = fs::read_to_string(&file_a).unwrap();
        let mut visited = HashSet::new();

        let result = resolver.resolve(&content, &file_a, &mut visited, 0);
        assert!(result.is_err());
        match result {
            Err(RenderError::CircularInclude { .. }) => {}
            _ => panic!("Expected CircularInclude error"),
        }
    }

    #[test]
    fn test_depth_limit() {
        let dir = tempdir().unwrap();

        // Create a deep chain: 0 -> 1 -> 2 -> ... -> 10
        for i in 0..10 {
            let file = dir.path().join(format!("{}.txt", i));
            if i < 9 {
                let content = format!("Level {}\n{{{{> {}.txt }}}}", i, i + 1)
                    .replace("{{{{", "{{")
                    .replace("}}}}", "}}");
                fs::write(&file, content).unwrap();
            } else {
                fs::write(&file, format!("Level {}", i)).unwrap();
            }
        }

        let main_file = dir.path().join("0.txt");
        let content = fs::read_to_string(&main_file).unwrap();

        // With depth limit 5, should fail
        let resolver = IncludeResolver::new(dir.path(), 5);
        let mut visited = HashSet::new();
        let result = resolver.resolve(&content, &main_file, &mut visited, 0);
        assert!(result.is_err());
        match result {
            Err(RenderError::IncludeDepthExceeded { .. }) => {}
            _ => panic!("Expected IncludeDepthExceeded error"),
        }
    }

    #[test]
    fn test_path_traversal_prevention() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        // Create a file outside the root
        let outside_file = dir.path().join("outside.txt");
        fs::write(&outside_file, "Outside content").unwrap();

        // Try to include using ../
        let main_file = subdir.join("main.txt");
        fs::write(&main_file, "{{> ../outside.txt }}").unwrap();

        // Root is subdir, so ../outside.txt should be forbidden
        let resolver = IncludeResolver::new(&subdir, 20);
        let content = fs::read_to_string(&main_file).unwrap();
        let mut visited = HashSet::new();

        let result = resolver.resolve(&content, &main_file, &mut visited, 0);
        assert!(result.is_err());
        match result {
            Err(RenderError::PathTraversal { .. }) => {}
            _ => panic!("Expected PathTraversal error"),
        }
    }

    #[test]
    fn test_multiple_includes() {
        let dir = tempdir().unwrap();

        let file_a = dir.path().join("a.txt");
        let file_b = dir.path().join("b.txt");
        fs::write(&file_a, "Content A").unwrap();
        fs::write(&file_b, "Content B").unwrap();

        let main_file = dir.path().join("main.txt");
        fs::write(&main_file, "{{> a.txt }} and {{> b.txt }}").unwrap();

        let resolver = IncludeResolver::new(dir.path(), 20);
        let content = fs::read_to_string(&main_file).unwrap();
        let mut visited = HashSet::new();

        let result = resolver.resolve(&content, &main_file, &mut visited, 0).unwrap();
        assert_eq!(result, "Content A and Content B");
    }

    #[test]
    fn test_no_includes() {
        let dir = tempdir().unwrap();
        let main_file = dir.path().join("main.txt");
        fs::write(&main_file, "No includes here!").unwrap();

        let resolver = IncludeResolver::new(dir.path(), 20);
        let content = fs::read_to_string(&main_file).unwrap();
        let mut visited = HashSet::new();

        let result = resolver.resolve(&content, &main_file, &mut visited, 0).unwrap();
        assert_eq!(result, "No includes here!");
    }
}
