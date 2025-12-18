use crate::error::RenderError;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::include::IncludeResolver;
use super::variable::VariableSubstitutor;

pub struct TemplateEngine {
    root_dir: PathBuf,
    max_depth: usize,
    strict: bool,
    warn_undefined: bool,
}

impl TemplateEngine {
    pub fn new(root_dir: PathBuf, max_depth: usize, strict: bool, warn_undefined: bool) -> Self {
        Self {
            root_dir,
            max_depth,
            strict,
            warn_undefined,
        }
    }

    /// Render a template with the given data
    ///
    /// Processing order (as specified):
    /// 1. Load template
    /// 2. Resolve includes (recursively)
    /// 3. Substitute variables (once)
    /// 4. Unescape \{{ -> {{
    pub fn render(&self, template_path: &Path, data: &Value) -> Result<String, RenderError> {
        // 1. Load template
        let content =
            fs::read_to_string(template_path).map_err(|e| RenderError::TemplateFileRead {
                path: template_path.display().to_string(),
                source: e,
            })?;

        // 2. Resolve includes
        let include_resolver = IncludeResolver::new(&self.root_dir, self.max_depth);
        let mut visited = HashSet::new();
        let expanded = include_resolver.resolve(&content, template_path, &mut visited, 0)?;

        // 3. Substitute variables
        let variable_substitutor = VariableSubstitutor::new(self.strict, self.warn_undefined);
        let substituted = variable_substitutor.substitute(&expanded, data)?;

        // 4. Unescape \{{ -> {{
        // This is already handled in the VariableSubstitutor, so we just return
        Ok(substituted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_simple_render() {
        let dir = tempdir().unwrap();
        let template = dir.path().join("template.txt");
        fs::write(&template, "Hello, {{ name }}!").unwrap();

        let data = json!({"name": "World"});
        let engine = TemplateEngine::new(dir.path().to_path_buf(), 20, false, false);
        let result = engine.render(&template, &data).unwrap();

        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_with_include() {
        let dir = tempdir().unwrap();

        let header = dir.path().join("header.txt");
        fs::write(&header, "=== {{ title }} ===").unwrap();

        let template = dir.path().join("template.txt");
        fs::write(&template, "{{> header.txt }}\nContent: {{ content }}").unwrap();

        let data = json!({"title": "My Title", "content": "My Content"});
        let engine = TemplateEngine::new(dir.path().to_path_buf(), 20, false, false);
        let result = engine.render(&template, &data).unwrap();

        assert_eq!(result, "=== My Title ===\nContent: My Content");
    }

    #[test]
    fn test_nested_include_with_variables() {
        let dir = tempdir().unwrap();

        let footer = dir.path().join("footer.txt");
        fs::write(&footer, "Footer: {{ footer_text }}").unwrap();

        let body = dir.path().join("body.txt");
        fs::write(&body, "Body: {{ body_text }}\n{{> footer.txt }}").unwrap();

        let template = dir.path().join("template.txt");
        fs::write(&template, "Header: {{ header_text }}\n{{> body.txt }}").unwrap();

        let data = json!({
            "header_text": "Top",
            "body_text": "Middle",
            "footer_text": "Bottom"
        });

        let engine = TemplateEngine::new(dir.path().to_path_buf(), 20, false, false);
        let result = engine.render(&template, &data).unwrap();

        assert_eq!(result, "Header: Top\nBody: Middle\nFooter: Bottom");
    }

    #[test]
    fn test_escape_in_included_file() {
        let dir = tempdir().unwrap();

        let included = dir.path().join("included.txt");
        fs::write(&included, r"Use \{{ variable }} for variables").unwrap();

        let template = dir.path().join("template.txt");
        fs::write(&template, "{{> included.txt }}").unwrap();

        let data = json!({});
        let engine = TemplateEngine::new(dir.path().to_path_buf(), 20, false, false);
        let result = engine.render(&template, &data).unwrap();

        assert_eq!(result, "Use {{ variable }} for variables");
    }

    #[test]
    fn test_undefined_variable_strict() {
        let dir = tempdir().unwrap();
        let template = dir.path().join("template.txt");
        fs::write(&template, "Hello, {{ undefined }}!").unwrap();

        let data = json!({});
        let engine = TemplateEngine::new(dir.path().to_path_buf(), 20, true, false);
        let result = engine.render(&template, &data);

        assert!(result.is_err());
    }

    #[test]
    fn test_undefined_variable_non_strict() {
        let dir = tempdir().unwrap();
        let template = dir.path().join("template.txt");
        fs::write(&template, "Hello, {{ undefined }}!").unwrap();

        let data = json!({});
        let engine = TemplateEngine::new(dir.path().to_path_buf(), 20, false, false);
        let result = engine.render(&template, &data).unwrap();

        assert_eq!(result, "Hello, !");
    }

    #[test]
    fn test_complex_scenario() {
        let dir = tempdir().unwrap();

        // Create a partial for segments
        let segment = dir.path().join("segment.txt");
        fs::write(
            &segment,
            "- {{ name }}: {{ description }}",
        )
        .unwrap();

        // Create main template
        let template = dir.path().join("template.txt");
        fs::write(
            &template,
            r"# {{ title }}

Segments:
{{> segment.txt }}

Use \{{ var }} to reference variables.",
        )
        .unwrap();

        let data = json!({
            "title": "My Document",
            "name": "Engineers",
            "description": "Backend developers"
        });

        let engine = TemplateEngine::new(dir.path().to_path_buf(), 20, false, false);
        let result = engine.render(&template, &data).unwrap();

        let expected = r"# My Document

Segments:
- Engineers: Backend developers

Use {{ var }} to reference variables.";

        assert_eq!(result, expected);
    }
}
