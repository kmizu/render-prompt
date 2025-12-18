use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "render-prompt",
    version,
    about = "Template engine with variable substitution and include functionality",
    long_about = "A minimal template engine that renders templates with variable substitution ({{ var }}) \
                  and include directives ({{> file }}). Supports YAML and JSON data sources."
)]
pub struct Cli {
    /// Template file path
    #[arg(short = 't', long = "template", required = true, value_name = "PATH")]
    pub template: String,

    /// Data files (YAML/JSON). Can be specified multiple times.
    /// Multiple files will be deep-merged with later files taking precedence.
    #[arg(short = 'd', long = "data", value_name = "PATH")]
    pub data: Vec<String>,

    /// Output file path. If not specified, output goes to stdout.
    #[arg(short = 'o', long = "out", value_name = "PATH")]
    pub output: Option<String>,

    /// Root directory for include resolution.
    /// If not specified, uses the template file's directory.
    #[arg(short = 'r', long = "root", value_name = "DIR")]
    pub root: Option<String>,

    /// Strict mode: treat undefined variables as errors
    #[arg(long = "strict")]
    pub strict: bool,

    /// Warn on undefined variables (writes warnings to stderr)
    #[arg(long = "warn-undefined")]
    pub warn_undefined: bool,

    /// Maximum include depth to prevent infinite recursion
    #[arg(long = "max-include-depth", value_name = "N", default_value = "20")]
    pub max_include_depth: usize,

    /// Print dependency tree (all template files) and exit
    #[arg(long = "print-deps")]
    pub print_deps: bool,
}

impl Cli {
    /// Validate CLI arguments
    pub fn validate(&self) -> Result<(), String> {
        // Check if template file path is provided (already enforced by required = true)

        // Check max_include_depth is reasonable
        if self.max_include_depth == 0 {
            return Err("max-include-depth must be at least 1".to_string());
        }

        if self.max_include_depth > 1000 {
            return Err("max-include-depth is too large (max: 1000)".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_max_depth_zero() {
        let cli = Cli {
            template: "test.txt".to_string(),
            data: vec![],
            output: None,
            root: None,
            strict: false,
            warn_undefined: false,
            max_include_depth: 0,
            print_deps: false,
        };

        assert!(cli.validate().is_err());
    }

    #[test]
    fn test_validate_max_depth_too_large() {
        let cli = Cli {
            template: "test.txt".to_string(),
            data: vec![],
            output: None,
            root: None,
            strict: false,
            warn_undefined: false,
            max_include_depth: 1001,
            print_deps: false,
        };

        assert!(cli.validate().is_err());
    }

    #[test]
    fn test_validate_ok() {
        let cli = Cli {
            template: "test.txt".to_string(),
            data: vec![],
            output: None,
            root: None,
            strict: false,
            warn_undefined: false,
            max_include_depth: 20,
            print_deps: false,
        };

        assert!(cli.validate().is_ok());
    }
}
