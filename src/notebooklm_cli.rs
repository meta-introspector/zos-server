// NotebookLM CLI for ZOS Integration
use crate::notebooklm_interface::NotebookLMInterface;
use std::fs;

pub struct NotebookLMCLI {
    interface: NotebookLMInterface,
}

impl NotebookLMCLI {
    pub fn new() -> Self {
        Self {
            interface: NotebookLMInterface::new(),
        }
    }

    pub fn import_file(&mut self, file_path: &str, topic: &str) -> Result<String, String> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;

        if content.len() > 2 * 1024 * 1024 {
            return Err(format!("File {} exceeds 2MB limit", file_path));
        }

        let chunk_id = self
            .interface
            .create_intelligent_chunk(content, topic.to_string())?;
        println!("📥 Created chunk {} from {}", chunk_id, file_path);

        Ok(chunk_id)
    }

    pub fn import_directory(&mut self, dir_path: &str) -> Result<Vec<String>, String> {
        let mut chunk_ids = Vec::new();

        let entries = fs::read_dir(dir_path)
            .map_err(|e| format!("Failed to read directory {}: {}", dir_path, e))?;

        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.ends_with(".txt")
                        || file_name.ends_with(".md")
                        || file_name.ends_with(".rs")
                    {
                        let topic = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown");

                        match self.import_file(path.to_str().unwrap(), topic) {
                            Ok(chunk_id) => chunk_ids.push(chunk_id),
                            Err(e) => println!("⚠️ Skipped {}: {}", file_name, e),
                        }
                    }
                }
            }
        }

        Ok(chunk_ids)
    }

    pub fn process_all(&mut self) -> Result<usize, String> {
        self.interface.batch_import()
    }

    pub fn show_stats(&self) {
        let stats = self.interface.get_chunk_stats();
        println!("📊 NotebookLM Import Stats:");
        for (key, value) in stats {
            println!("  {}: {}", key, value);
        }
    }
}

// CLI command handlers
pub fn handle_notebooklm_command(args: &[String]) -> Result<(), String> {
    let mut cli = NotebookLMCLI::new();

    match args.get(0).map(|s| s.as_str()) {
        Some("import") => {
            let file_path = args.get(1).ok_or("Missing file path")?;
            let default_topic = "general".to_string();
            let topic = args.get(2).unwrap_or(&default_topic).as_str();
            let chunk_id = cli.import_file(file_path, topic)?;
            println!("✅ Imported as chunk: {}", chunk_id);
        }
        Some("import-dir") => {
            let dir_path = args.get(1).ok_or("Missing directory path")?;
            let chunk_ids = cli.import_directory(dir_path)?;
            println!("✅ Imported {} files as chunks", chunk_ids.len());
        }
        Some("process") => {
            let count = cli.process_all()?;
            println!("✅ Processed {} chunks into ZOS", count);
        }
        Some("stats") => {
            cli.show_stats();
        }
        _ => {
            println!("NotebookLM Commands:");
            println!("  import <file> [topic]     - Import single file");
            println!("  import-dir <directory>    - Import all files in directory");
            println!("  process                   - Process all queued chunks");
            println!("  stats                     - Show import statistics");
        }
    }

    Ok(())
}
