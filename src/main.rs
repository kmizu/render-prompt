mod cli;
mod data;
mod error;
mod template;

use clap::Parser;
use cli::Cli;
use error::{RenderError, EXIT_SUCCESS};

fn main() {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Validate arguments
    if let Err(e) = cli.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(error::EXIT_USAGE_ERROR);
    }

    // Run the main logic
    match run(cli) {
        Ok(output) => {
            println!("{}", output);
            std::process::exit(EXIT_SUCCESS);
        }
        Err(e) => {
            // Print machine-readable error message to stderr
            eprintln!("{}", e.format_machine_readable());
            eprintln!("{}", e);
            std::process::exit(e.exit_code());
        }
    }
}

fn run(cli: Cli) -> Result<String, RenderError> {
    use data::DataLoader;
    use std::path::PathBuf;
    use template::TemplateEngine;

    // 1. Load and merge data files
    let data = if cli.data.is_empty() {
        serde_json::json!({})
    } else {
        DataLoader::load_multiple(&cli.data)?
    };

    // 2. Determine root directory
    let template_path = PathBuf::from(&cli.template);
    let root_dir = if let Some(root) = cli.root {
        PathBuf::from(root)
    } else {
        // Use template's parent directory as root
        template_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf()
    };

    // 3. Create template engine
    let engine = TemplateEngine::new(root_dir, cli.max_include_depth, cli.strict, cli.warn_undefined);

    // 4. Render template
    let output = engine.render(&template_path, &data)?;

    // 5. Write output
    if let Some(out_path) = cli.output {
        std::fs::write(&out_path, &output).map_err(|e| RenderError::Io(e))?;
        // Return empty string to avoid printing to stdout
        Ok(String::new())
    } else {
        // Return output for stdout
        Ok(output)
    }
}
