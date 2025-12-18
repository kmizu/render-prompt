use crate::error::{Location, RenderError};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;

lazy_static! {
    // Match {{ var }} or \{{ (escaped)
    // Group 1: optional backslash for escape
    // Group 2: variable name/path
    static ref VAR_PATTERN: Regex = Regex::new(r"(\\)?\{\{\s*([^}]+?)\s*\}\}").unwrap();
}

pub struct VariableSubstitutor {
    strict: bool,
    warn_undefined: bool,
}

impl VariableSubstitutor {
    pub fn new(strict: bool, warn_undefined: bool) -> Self {
        Self {
            strict,
            warn_undefined,
        }
    }

    /// Substitute all variables in the content
    pub fn substitute(&self, content: &str, data: &Value) -> Result<String, RenderError> {
        let mut result = String::with_capacity(content.len());
        let mut last_end = 0;

        for cap in VAR_PATTERN.captures_iter(content) {
            let full_match = cap.get(0).unwrap();
            let start = full_match.start();
            let end = full_match.end();

            // Add text before this match
            result.push_str(&content[last_end..start]);

            // Check if this is escaped
            if cap.get(1).is_some() {
                // Escaped: \{{ ... }} -> {{ ... }}
                result.push_str("{{");
                if let Some(var_name) = cap.get(2) {
                    result.push(' ');
                    result.push_str(var_name.as_str());
                    result.push(' ');
                }
                result.push_str("}}");
            } else {
                // Not escaped: perform substitution
                let var_path = cap.get(2).unwrap().as_str().trim();
                let location = Location::from_offset(content, start, "<template>");

                match self.resolve_variable(var_path, data, location.clone()) {
                    Ok(value) => result.push_str(&value),
                    Err(e) => {
                        if self.strict {
                            return Err(e);
                        } else {
                            if self.warn_undefined {
                                eprintln!(
                                    "Warning: undefined variable '{}' at {}",
                                    var_path, location
                                );
                            }
                            // In non-strict mode, replace with empty string
                        }
                    }
                }
            }

            last_end = end;
        }

        // Add remaining text
        result.push_str(&content[last_end..]);

        Ok(result)
    }

    /// Resolve a variable path like "user.name" or "items.0"
    fn resolve_variable(
        &self,
        path: &str,
        data: &Value,
        location: Location,
    ) -> Result<String, RenderError> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = data;

        for (i, part) in parts.iter().enumerate() {
            // Try to parse as array index first
            if let Ok(index) = part.parse::<usize>() {
                if let Some(value) = current.get(index) {
                    current = value;
                    continue;
                }
                // If array index fails, try as object key
            }

            // Treat as object key
            current = current.get(part).ok_or_else(|| RenderError::UndefinedVariable {
                name: path.to_string(),
                location: location.clone(),
            })?;
        }

        // Convert Value to String
        Ok(Self::value_to_string(current))
    }

    /// Convert a JSON value to its string representation
    fn value_to_string(value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => String::new(),
            Value::Array(_) | Value::Object(_) => {
                // Convert complex types to JSON string
                serde_json::to_string(value).unwrap_or_else(|_| String::from("{}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_substitution() {
        let sub = VariableSubstitutor::new(false, false);
        let data = json!({"name": "Alice"});
        let result = sub.substitute("Hello, {{ name }}!", &data).unwrap();
        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_whitespace_trimming() {
        let sub = VariableSubstitutor::new(false, false);
        let data = json!({"name": "Bob"});
        let result = sub.substitute("Hello, {{  name  }}!", &data).unwrap();
        assert_eq!(result, "Hello, Bob!");
    }

    #[test]
    fn test_nested_path() {
        let sub = VariableSubstitutor::new(false, false);
        let data = json!({
            "user": {
                "profile": {
                    "name": "Charlie"
                }
            }
        });
        let result = sub
            .substitute("Name: {{ user.profile.name }}", &data)
            .unwrap();
        assert_eq!(result, "Name: Charlie");
    }

    #[test]
    fn test_array_access() {
        let sub = VariableSubstitutor::new(false, false);
        let data = json!({
            "items": ["apple", "banana", "cherry"]
        });
        let result = sub.substitute("First: {{ items.0 }}", &data).unwrap();
        assert_eq!(result, "First: apple");
    }

    #[test]
    fn test_array_access_deep() {
        let sub = VariableSubstitutor::new(false, false);
        let data = json!({
            "matrix": [
                ["a", "b"],
                ["c", "d"]
            ]
        });
        let result = sub.substitute("Value: {{ matrix.1.0 }}", &data).unwrap();
        assert_eq!(result, "Value: c");
    }

    #[test]
    fn test_number_value() {
        let sub = VariableSubstitutor::new(false, false);
        let data = json!({"age": 30});
        let result = sub.substitute("Age: {{ age }}", &data).unwrap();
        assert_eq!(result, "Age: 30");
    }

    #[test]
    fn test_boolean_value() {
        let sub = VariableSubstitutor::new(false, false);
        let data = json!({"active": true, "inactive": false});
        let result = sub
            .substitute("Active: {{ active }}, Inactive: {{ inactive }}", &data)
            .unwrap();
        assert_eq!(result, "Active: true, Inactive: false");
    }

    #[test]
    fn test_null_value() {
        let sub = VariableSubstitutor::new(false, false);
        let data = json!({"empty": null});
        let result = sub.substitute("Empty: {{ empty }}", &data).unwrap();
        assert_eq!(result, "Empty: ");
    }

    #[test]
    fn test_object_to_json() {
        let sub = VariableSubstitutor::new(false, false);
        let data = json!({"obj": {"key": "value"}});
        let result = sub.substitute("Obj: {{ obj }}", &data).unwrap();
        assert_eq!(result, r#"Obj: {"key":"value"}"#);
    }

    #[test]
    fn test_array_to_json() {
        let sub = VariableSubstitutor::new(false, false);
        let data = json!({"arr": [1, 2, 3]});
        let result = sub.substitute("Arr: {{ arr }}", &data).unwrap();
        assert_eq!(result, "Arr: [1,2,3]");
    }

    #[test]
    fn test_escape_simple() {
        let sub = VariableSubstitutor::new(false, false);
        let data = json!({"name": "Alice"});
        let result = sub.substitute(r"\{{ name }}", &data).unwrap();
        assert_eq!(result, "{{ name }}");
    }

    #[test]
    fn test_escape_mixed() {
        let sub = VariableSubstitutor::new(false, false);
        let data = json!({"name": "Alice"});
        let result = sub
            .substitute(r"Hello {{ name }}, use \{{ variable }}", &data)
            .unwrap();
        assert_eq!(result, "Hello Alice, use {{ variable }}");
    }

    #[test]
    fn test_undefined_variable_non_strict() {
        let sub = VariableSubstitutor::new(false, false);
        let data = json!({});
        let result = sub.substitute("Hello {{ undefined }}!", &data).unwrap();
        assert_eq!(result, "Hello !");
    }

    #[test]
    fn test_undefined_variable_strict() {
        let sub = VariableSubstitutor::new(true, false);
        let data = json!({});
        let result = sub.substitute("Hello {{ undefined }}!", &data);
        assert!(result.is_err());
        match result {
            Err(RenderError::UndefinedVariable { name, .. }) => {
                assert_eq!(name, "undefined");
            }
            _ => panic!("Expected UndefinedVariable error"),
        }
    }

    #[test]
    fn test_multiple_substitutions() {
        let sub = VariableSubstitutor::new(false, false);
        let data = json!({"first": "Alice", "last": "Smith"});
        let result = sub
            .substitute("Name: {{ first }} {{ last }}", &data)
            .unwrap();
        assert_eq!(result, "Name: Alice Smith");
    }

    #[test]
    fn test_no_substitution() {
        let sub = VariableSubstitutor::new(false, false);
        let data = json!({});
        let result = sub.substitute("No variables here!", &data).unwrap();
        assert_eq!(result, "No variables here!");
    }
}
