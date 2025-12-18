use serde_json::Value;

/// Deep merge two JSON values
/// Later values take precedence over earlier values (last-wins)
/// Arrays are replaced entirely, not merged
pub struct DataMerger;

impl DataMerger {
    /// Merge `overlay` into `base`, modifying `base` in place
    ///
    /// Rules:
    /// - If both are objects: recursively merge keys (overlay wins on conflict)
    /// - If both are arrays: overlay completely replaces base
    /// - Otherwise: overlay replaces base
    pub fn merge(base: &mut Value, overlay: &Value) {
        match (base, overlay) {
            // Both are objects: deep merge
            (Value::Object(base_map), Value::Object(overlay_map)) => {
                for (key, overlay_value) in overlay_map {
                    if let Some(base_value) = base_map.get_mut(key) {
                        // Key exists in both: recursively merge
                        Self::merge(base_value, overlay_value);
                    } else {
                        // Key only in overlay: insert it
                        base_map.insert(key.clone(), overlay_value.clone());
                    }
                }
            }
            // Not both objects: overlay wins
            (base, overlay) => {
                *base = overlay.clone();
            }
        }
    }

    /// Merge multiple values from left to right
    /// Returns the merged result
    pub fn merge_multiple(values: Vec<Value>) -> Value {
        if values.is_empty() {
            return Value::Object(serde_json::Map::new());
        }

        let mut result = values[0].clone();
        for value in values.iter().skip(1) {
            Self::merge(&mut result, value);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_simple() {
        let mut base = json!({ "a": 1 });
        let overlay = json!({ "b": 2 });
        DataMerger::merge(&mut base, &overlay);
        assert_eq!(base, json!({ "a": 1, "b": 2 }));
    }

    #[test]
    fn test_merge_override() {
        let mut base = json!({ "a": 1 });
        let overlay = json!({ "a": 2 });
        DataMerger::merge(&mut base, &overlay);
        assert_eq!(base, json!({ "a": 2 }));
    }

    #[test]
    fn test_merge_deep() {
        let mut base = json!({
            "user": {
                "name": "Alice",
                "age": 25
            }
        });
        let overlay = json!({
            "user": {
                "age": 30,
                "city": "Tokyo"
            }
        });
        DataMerger::merge(&mut base, &overlay);
        assert_eq!(
            base,
            json!({
                "user": {
                    "name": "Alice",
                    "age": 30,
                    "city": "Tokyo"
                }
            })
        );
    }

    #[test]
    fn test_merge_array_replace() {
        let mut base = json!({ "items": [1, 2, 3] });
        let overlay = json!({ "items": [4, 5] });
        DataMerger::merge(&mut base, &overlay);
        assert_eq!(base, json!({ "items": [4, 5] }));
    }

    #[test]
    fn test_merge_type_change() {
        let mut base = json!({ "value": "string" });
        let overlay = json!({ "value": 42 });
        DataMerger::merge(&mut base, &overlay);
        assert_eq!(base, json!({ "value": 42 }));
    }

    #[test]
    fn test_merge_nested_objects() {
        let mut base = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "key": "value1"
                    }
                }
            }
        });
        let overlay = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "key": "value2",
                        "new_key": "new_value"
                    }
                }
            }
        });
        DataMerger::merge(&mut base, &overlay);
        assert_eq!(
            base,
            json!({
                "level1": {
                    "level2": {
                        "level3": {
                            "key": "value2",
                            "new_key": "new_value"
                        }
                    }
                }
            })
        );
    }

    #[test]
    fn test_merge_multiple_empty() {
        let result = DataMerger::merge_multiple(vec![]);
        assert_eq!(result, json!({}));
    }

    #[test]
    fn test_merge_multiple_single() {
        let result = DataMerger::merge_multiple(vec![json!({"a": 1})]);
        assert_eq!(result, json!({"a": 1}));
    }

    #[test]
    fn test_merge_multiple_three() {
        let result = DataMerger::merge_multiple(vec![
            json!({"a": 1, "b": 2}),
            json!({"b": 3, "c": 4}),
            json!({"c": 5, "d": 6}),
        ]);
        assert_eq!(result, json!({"a": 1, "b": 3, "c": 5, "d": 6}));
    }
}
