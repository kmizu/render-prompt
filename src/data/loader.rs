use crate::error::RenderError;
use serde_json::Value;
use std::fs;
use std::path::Path;

use super::merger::DataMerger;

pub struct DataLoader;

impl DataLoader {
    /// Load a single data file (YAML or JSON)
    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Value, RenderError> {
        let path = path.as_ref();
        let path_str = path.display().to_string();

        // Read file content
        let content = fs::read_to_string(path).map_err(|e| RenderError::DataFileRead {
            path: path_str.clone(),
            source: e,
        })?;

        // Determine format from extension
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "yaml" | "yml" => {
                // Parse as YAML
                serde_yaml::from_str(&content).map_err(|e| RenderError::DataFileParse {
                    path: path_str,
                    source: anyhow::Error::new(e),
                })
            }
            "json" => {
                // Parse as JSON
                serde_json::from_str(&content).map_err(|e| RenderError::DataFileParse {
                    path: path_str,
                    source: anyhow::Error::new(e),
                })
            }
            _ => Err(RenderError::DataFileParse {
                path: path_str,
                source: anyhow::anyhow!(
                    "Unsupported file extension: '{}'. Expected .yaml, .yml, or .json",
                    extension
                ),
            }),
        }
    }

    /// Load multiple data files and merge them (later files override earlier ones)
    pub fn load_multiple<P: AsRef<Path>>(paths: &[P]) -> Result<Value, RenderError> {
        if paths.is_empty() {
            // Return empty object if no data files provided
            return Ok(Value::Object(serde_json::Map::new()));
        }

        let mut values = Vec::new();
        for path in paths {
            let value = Self::load_file(path)?;
            values.push(value);
        }

        Ok(DataMerger::merge_multiple(values))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_json() {
        let mut file = NamedTempFile::with_suffix(".json").unwrap();
        writeln!(file, r#"{{"name": "Alice", "age": 30}}"#).unwrap();

        let result = DataLoader::load_file(file.path()).unwrap();
        assert_eq!(result, json!({"name": "Alice", "age": 30}));
    }

    #[test]
    fn test_load_yaml() {
        let mut file = NamedTempFile::with_suffix(".yaml").unwrap();
        writeln!(file, "name: Bob").unwrap();
        writeln!(file, "age: 25").unwrap();

        let result = DataLoader::load_file(file.path()).unwrap();
        assert_eq!(result, json!({"name": "Bob", "age": 25}));
    }

    #[test]
    fn test_load_yml_extension() {
        let mut file = NamedTempFile::with_suffix(".yml").unwrap();
        writeln!(file, "key: value").unwrap();

        let result = DataLoader::load_file(file.path()).unwrap();
        assert_eq!(result, json!({"key": "value"}));
    }

    #[test]
    fn test_load_invalid_extension() {
        let mut file = NamedTempFile::with_suffix(".txt").unwrap();
        writeln!(file, "some text").unwrap();

        let result = DataLoader::load_file(file.path());
        assert!(result.is_err());
        match result {
            Err(RenderError::DataFileParse { .. }) => {}
            _ => panic!("Expected DataFileParse error"),
        }
    }

    #[test]
    fn test_load_invalid_json() {
        let mut file = NamedTempFile::with_suffix(".json").unwrap();
        writeln!(file, "{{invalid json}}").unwrap();

        let result = DataLoader::load_file(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = DataLoader::load_file("/nonexistent/path/file.json");
        assert!(result.is_err());
        match result {
            Err(RenderError::DataFileRead { .. }) => {}
            _ => panic!("Expected DataFileRead error"),
        }
    }

    #[test]
    fn test_load_multiple_empty() {
        let paths: Vec<String> = vec![];
        let result = DataLoader::load_multiple(&paths).unwrap();
        assert_eq!(result, json!({}));
    }

    #[test]
    fn test_load_multiple_merge() {
        let mut file1 = NamedTempFile::with_suffix(".json").unwrap();
        writeln!(file1, r#"{{"a": 1, "b": 2}}"#).unwrap();

        let mut file2 = NamedTempFile::with_suffix(".json").unwrap();
        writeln!(file2, r#"{{"b": 3, "c": 4}}"#).unwrap();

        let paths = vec![file1.path(), file2.path()];
        let result = DataLoader::load_multiple(&paths).unwrap();

        // Later file wins on conflict (b: 3, not 2)
        assert_eq!(result, json!({"a": 1, "b": 3, "c": 4}));
    }

    #[test]
    fn test_load_multiple_yaml_and_json() {
        let mut file1 = NamedTempFile::with_suffix(".yaml").unwrap();
        writeln!(file1, "x: 10").unwrap();

        let mut file2 = NamedTempFile::with_suffix(".json").unwrap();
        writeln!(file2, r#"{{"y": 20}}"#).unwrap();

        let paths = vec![file1.path(), file2.path()];
        let result = DataLoader::load_multiple(&paths).unwrap();

        assert_eq!(result, json!({"x": 10, "y": 20}));
    }
}
