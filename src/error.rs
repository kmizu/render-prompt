use std::fmt;

/// Exit codes as defined in the specification
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_USAGE_ERROR: i32 = 2;
pub const EXIT_TEMPLATE_ERROR: i32 = 3;
pub const EXIT_DATA_ERROR: i32 = 4;
pub const EXIT_INCLUDE_ERROR: i32 = 5;
pub const EXIT_VARIABLE_ERROR: i32 = 6;
pub const EXIT_CIRCULAR_OR_DEPTH_ERROR: i32 = 7;

/// Location information for error reporting
#[derive(Debug, Clone)]
pub struct Location {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl Location {
    pub fn new(file: String, line: usize, column: usize) -> Self {
        Self { file, line, column }
    }

    pub fn unknown() -> Self {
        Self {
            file: "<unknown>".to_string(),
            line: 0,
            column: 0,
        }
    }

    /// Calculate location from content and byte offset
    pub fn from_offset(content: &str, offset: usize, file: &str) -> Self {
        let before = &content[..offset.min(content.len())];
        let lines: Vec<&str> = before.split('\n').collect();
        let line = lines.len();
        let column = lines.last().map(|l| l.len() + 1).unwrap_or(1);

        Location {
            file: file.to_string(),
            line,
            column,
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 && self.column == 0 {
            write!(f, "{}", self.file)
        } else {
            write!(f, "{}:{}:{}", self.file, self.line, self.column)
        }
    }
}

/// Main error type for render-prompt
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    // Data loading errors
    #[error("Failed to read data file '{path}': {source}")]
    DataFileRead {
        path: String,
        source: std::io::Error,
    },

    #[error("Failed to parse data file '{path}': {source}")]
    DataFileParse {
        path: String,
        source: anyhow::Error,
    },

    #[error("Data merge error: {0}")]
    DataMerge(String),

    // Template loading errors
    #[error("Failed to read template file '{path}': {source}")]
    TemplateFileRead {
        path: String,
        source: std::io::Error,
    },

    // Variable errors
    #[error("Undefined variable '{name}' at {location}")]
    UndefinedVariable { name: String, location: Location },

    #[error("Variable resolution error at {location}: {message}")]
    VariableResolution { message: String, location: Location },

    // Include errors
    #[error("Failed to read included file '{path}': {source}")]
    IncludeFileRead {
        path: String,
        source: std::io::Error,
    },

    #[error("Include not found: '{path}' referenced from {from}")]
    IncludeNotFound { path: String, from: String },

    #[error("Path traversal attempt detected: '{path}' is outside root directory")]
    PathTraversal { path: String },

    #[error("Circular include detected: {path}")]
    CircularInclude { path: String },

    #[error("Include depth limit exceeded (max: {max_depth})")]
    IncludeDepthExceeded { max_depth: usize },

    // Generic I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    // CLI usage error
    #[error("Usage error: {0}")]
    Usage(String),
}

impl RenderError {
    /// Get the appropriate exit code for this error
    pub fn exit_code(&self) -> i32 {
        match self {
            RenderError::Usage(_) => EXIT_USAGE_ERROR,
            RenderError::TemplateFileRead { .. } => EXIT_TEMPLATE_ERROR,
            RenderError::DataFileRead { .. } | RenderError::DataFileParse { .. } | RenderError::DataMerge(_) => {
                EXIT_DATA_ERROR
            }
            RenderError::IncludeFileRead { .. }
            | RenderError::IncludeNotFound { .. }
            | RenderError::PathTraversal { .. } => EXIT_INCLUDE_ERROR,
            RenderError::UndefinedVariable { .. } | RenderError::VariableResolution { .. } => {
                EXIT_VARIABLE_ERROR
            }
            RenderError::CircularInclude { .. } | RenderError::IncludeDepthExceeded { .. } => {
                EXIT_CIRCULAR_OR_DEPTH_ERROR
            }
            RenderError::Io(_) => EXIT_INCLUDE_ERROR,
        }
    }

    /// Format error for machine-readable output
    pub fn format_machine_readable(&self) -> String {
        match self {
            RenderError::UndefinedVariable { name, location } => {
                format!(
                    "ERROR code=UNDEFINED_VAR var=\"{}\" template=\"{}\" line={} col={}",
                    name, location.file, location.line, location.column
                )
            }
            RenderError::IncludeNotFound { path, from } => {
                format!(
                    "ERROR code=INCLUDE_NOT_FOUND file=\"{}\" from=\"{}\"",
                    path, from
                )
            }
            RenderError::CircularInclude { path } => {
                format!("ERROR code=CIRCULAR_INCLUDE path=\"{}\"", path)
            }
            RenderError::PathTraversal { path } => {
                format!("ERROR code=PATH_TRAVERSAL path=\"{}\"", path)
            }
            RenderError::IncludeDepthExceeded { max_depth } => {
                format!("ERROR code=DEPTH_EXCEEDED max={}", max_depth)
            }
            _ => format!("ERROR: {}", self),
        }
    }
}
