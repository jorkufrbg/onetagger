#[macro_use] extern crate log;
#[macro_use] extern crate anyhow;
#[macro_use] extern crate lazy_static;

use std::path::{Path, PathBuf};
use anyhow::Error;
use onetagger_tagger::AudioFileInfo;
use serde::{Serialize, Deserialize};

pub mod ac;
pub mod docs;
pub mod parser;

// Re-export
pub use parser::{TemplateParser, SyntaxData, SyntaxType};


/// Renamer itself
pub struct Renamer {
    template: TemplateParser
}

impl Renamer {
    /// Create new instance
    pub fn new(template: TemplateParser) -> Renamer {
        Renamer { template }
    }

    /// Generate new filename
    pub fn generate_name(&mut self, output_dir: impl AsRef<Path>, info: &AudioFileInfo, config: &RenamerConfig) -> PathBuf {
        let mut name = self.template.evaluate(info, config);
        while name.starts_with("/") {
            name = name[1..].to_string()
        }
        if cfg!(windows) {
            name = name.replace("/", "\\");
        }
        let ext = info.path.extension().unwrap_or_default().to_string_lossy();
        output_dir.as_ref().join(format!("{name}.{ext}"))
    }


    /// Generate names - output: [(from, to),...]
    pub fn generate<I>(&mut self, files: I, config: &RenamerConfig) -> Result<Vec<(PathBuf, PathBuf)>, Error>
    where
        I: IntoIterator<Item = Result<AudioFileInfo, Error>>
    {
        let input_path = dunce::canonicalize(&config.path)?;
        if !input_path.exists() {
            return Err(anyhow!("Invalid path!"));
        }
        
        // Get output path
        let mut out_dir = config.out_dir.clone().unwrap_or(config.path.to_owned());
        if out_dir.to_string_lossy().trim().is_empty() {
            out_dir = config.path.to_owned();
        }

        let mut output = vec![];
        for file in files.into_iter() {
            // Load files
            let info = match file {
                Ok(i) => i,
                Err(e) => {
                    warn!("Failed loading file: {e}");
                    continue;
                },
            };
            let file = &info.path;

            // Get output dir
            let mut output_dir = Path::new(&out_dir).to_owned();
            if config.keep_subfolders {
                // Try to strip prefix and join with original or else fallback to parent
                output_dir = dunce::canonicalize(file)
                    .ok()
                    .map(|p| p.strip_prefix(&input_path).map(|p| p.parent().map(|p| p.to_owned())).ok().flatten())
                    .flatten()
                    .map(|p| output_dir.join(p))
                    .or_else(|| dunce::canonicalize(file)
                        .ok()
                        .map(|p| p.parent().map(|p| p.to_owned()))
                        .flatten()
                    )
                    .unwrap_or(output_dir);
            }

            let new_name = self.generate_name(output_dir, &info, config);
            output.push((file.to_owned(), new_name));
        }
        Ok(output)
    }

    /// Rename files, files = output from generate
    pub fn rename(&mut self, files: &[(PathBuf, PathBuf)], config: &RenamerConfig) -> Result<(), Error> {
        for (from, to) in files {
            // Don't overwrite
            if !config.overwrite && to.exists() {
                info!("File exists, skipping: {to:?}");
                continue;
            }

            // Create dir
            if let Some(parent) = to.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    warn!("Failed creating dir {parent:?}: {e}");
                }
            }
            // Copy mode
            if config.copy {
                match std::fs::copy(&from, &to) {
                    Ok(_) => info!("Copied: {to:?}"),
                    Err(e) => error!("Failed copying {from:?} -> {to:?}: {e}"),
                }
            // Move
            } else {
                if std::fs::rename(&from, &to).is_err() {
                    info!("Renaming failed, might be different FS, trying to copy.");
                    match std::fs::copy(&from, &to) {
                        Ok(_) => {
                            info!("Copied: {to:?}");
                            if let Err(e) = std::fs::remove_file(&from) {
                                warn!("Failed deleting {from:?}: {e}");
                            }
                        },
                        Err(e) => error!("Failed copying {from:?} -> {to:?}: {e}"),
                    }
                } else {
                    info!("Renamed: {to:?}");
                }
            }
        }

        Ok(())
    }

    /// Generate html from the syntax highlighting
    pub fn generate_html(&self, input: &str) -> String {
        // class prefix
        let prefix = "__renamer_";
        
        let mut output = String::new();
        for syntax in &self.template.syntax {
            let text = input.chars().skip(syntax.start).take(syntax.length).collect::<String>().replace(" ", "&nbsp;");
            let class = match syntax.syntax {
                SyntaxType::Text => "syntax_text",
                SyntaxType::String => "syntax_string",
                SyntaxType::Number => "syntax_number",
                SyntaxType::Function => "syntax_function",
                SyntaxType::Operator => "syntax_operator",
                SyntaxType::Property => "syntax_property",
                SyntaxType::Variable => "syntax_variable",
            };
            output.push_str(&format!("<span class=\"{prefix}{class}\">{text}</span>"));
        }
        output
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenamerConfig {
    pub path: PathBuf,
    pub out_dir: Option<PathBuf>,
    pub template: String,
    pub copy: bool,
    pub subfolders: bool,
    pub overwrite: bool,
    pub separator: String,
    pub keep_subfolders: bool,
}

impl RenamerConfig {
    /// Create new instance with default properties and paths
    pub fn default_with_paths(path: impl AsRef<Path>, template: &str) -> RenamerConfig {
        RenamerConfig {
            path: path.as_ref().to_path_buf(),
            out_dir: None,
            template: template.to_string(),
            copy: false,
            overwrite: false,
            keep_subfolders: false,
            separator: ", ".to_owned(),
            subfolders: true,
        }
    }
}

/// HTML generation test
#[test]
fn generate_html() {
    let items = [
        "%artist% - %title%",
        "%track%. %artist% - %title%",
        "%artist% - %title% - %bpm% - %key%",
        "%artist% - %album%/%track% - %title%",
        "%year% - %album%/%track% - %artist% - %title%"
    ];
    for i in items {
        let renamer = Renamer::new(TemplateParser::parse(i));
        let output = renamer.generate_html(i);
        println!("{output}")
    }
}
