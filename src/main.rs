use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    default_model: String,
    ollama_host: String,
    temperature: f32,
    stream: bool,
    system_prompt: String,
    enable_rag: bool,
    rag_max_files: usize,
    rag_max_file_size: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_model: String::from("llama3.2"),
            ollama_host: String::from("http://host.docker.internal:11434"),
            temperature: 0.7,
            stream: true,
            system_prompt: String::from(
                "You are a helpful and concise CLI assistant. Provide direct, to-the-point answers without unnecessary elaboration. \
                Focus on practical, actionable information. Use code examples when relevant. \
                Keep explanations brief unless asked for details."
            ),
            enable_rag: true,
            rag_max_files: 50,
            rag_max_file_size: 10240, // 10KB
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Debug, Deserialize)]
struct OllamaStreamResponse {
    response: String,
    done: bool,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "CLI tool for interacting with Ollama AI models from WSL", long_about = None)]
struct Args {
    /// Model to use (overrides default)
    #[arg(short, long)]
    model: Option<String>,

    /// Temperature for generation (0.0 to 1.0)
    #[arg(short, long)]
    temperature: Option<f32>,

    /// Set new default model
    #[arg(long)]
    set_default: Option<String>,

    /// List available models
    #[arg(short, long)]
    list: bool,

    /// Show current configuration
    #[arg(long)]
    config: bool,

    /// Reset configuration to defaults
    #[arg(long)]
    reset: bool,

    /// The prompt or command (if not provided, enters interactive mode)
    prompt: Vec<String>,
}

struct Assistant {
    config: Config,
    config_path: PathBuf,
    client: reqwest::blocking::Client,
}

impl Assistant {
    fn gather_directory_context(&self) -> Result<String> {
        if !self.config.enable_rag {
            return Ok(String::new());
        }

        let mut context = String::new();
        context.push_str("\n## Current Directory Context:\n\n");

        // Get current directory
        let current_dir = std::env::current_dir()?;
        context.push_str(&format!("Working directory: {}\n\n", current_dir.display()));

        // Collect file information
        let mut files = Vec::new();
        let mut total_files = 0;

        for entry in WalkDir::new(&current_dir)
            .max_depth(3)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') &&
                name != "target" &&
                name != "node_modules" &&
                name != "__pycache__" &&
                name != "dist" &&
                name != "build"
            })
        {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    total_files += 1;
                    if files.len() < self.config.rag_max_files {
                        if let Ok(metadata) = entry.metadata() {
                            if metadata.len() <= self.config.rag_max_file_size {
                                files.push(entry.path().to_path_buf());
                            }
                        }
                    }
                }
            }
        }

        // Add file tree structure
        context.push_str("### Project Structure (relevant files):\n");
        for file in &files {
            if let Ok(rel_path) = file.strip_prefix(&current_dir) {
                context.push_str(&format!("- {}\n", rel_path.display()));
            }
        }

        // Add content of important files
        context.push_str("\n### Important File Contents:\n");
        let important_files = ["README.md", "Cargo.toml", "package.json", "pyproject.toml", "go.mod"];

        for file in &files {
            let file_name = file.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if important_files.contains(&file_name) {
                if let Ok(content) = std::fs::read_to_string(file) {
                    if content.len() <= self.config.rag_max_file_size as usize {
                        if let Ok(rel_path) = file.strip_prefix(&current_dir) {
                            context.push_str(&format!("\n#### {}:\n```\n{}\n```\n",
                                rel_path.display(),
                                content.trim()
                            ));
                        }
                    }
                }
            }
        }

        if total_files > files.len() {
            context.push_str(&format!("\n(Showing {} of {} total files)\n", files.len(), total_files));
        }

        Ok(context)
    }

    fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Could not find config directory")?
            .join("assistant-cli");
        
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("config.toml");

        let config = if config_path.exists() {
            let config_str = std::fs::read_to_string(&config_path)?;
            toml::from_str(&config_str)?
        } else {
            let default_config = Config::default();
            let toml = toml::to_string(&default_config)?;
            std::fs::write(&config_path, toml)?;
            default_config
        };

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        Ok(Self {
            config,
            config_path,
            client,
        })
    }

    fn save_config(&self) -> Result<()> {
        let toml = toml::to_string(&self.config)?;
        std::fs::write(&self.config_path, toml)?;
        Ok(())
    }

    fn list_models(&self) -> Result<()> {
        let url = format!("{}/api/tags", self.config.ollama_host);
        
        let response = self.client.get(&url).send()?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch models: {}", response.status());
        }

        let data: serde_json::Value = response.json()?;
        
        println!("\n{}", "Available Models:".bold().cyan());
        println!("{}", "─".repeat(40).dimmed());
        
        if let Some(models) = data["models"].as_array() {
            for model in models {
                let name = model["name"].as_str().unwrap_or("unknown");
                let size = model["size"].as_u64().unwrap_or(0);
                let size_mb = size / (1024 * 1024);
                
                let is_default = name == self.config.default_model;
                let marker = if is_default { " ⭐" } else { "" };
                
                println!("  {} ({} MB){}", 
                    name.green(), 
                    size_mb.to_string().yellow(),
                    marker.bright_yellow()
                );
            }
        } else {
            println!("  {}", "No models found".red());
        }
        
        println!();
        Ok(())
    }

    fn show_config(&self) -> Result<()> {
        println!("\n{}", "Current Configuration:".bold().cyan());
        println!("{}", "─".repeat(40).dimmed());
        println!("  {}: {}", "Default Model".bold(), self.config.default_model.green());
        println!("  {}: {}", "Ollama Host".bold(), self.config.ollama_host.yellow());
        println!("  {}: {}", "Temperature".bold(), self.config.temperature.to_string().yellow());
        println!("  {}: {}", "Stream".bold(), self.config.stream.to_string().yellow());
        println!("  {}: {}", "Enable RAG".bold(), self.config.enable_rag.to_string().yellow());
        println!("  {}: {}", "RAG Max Files".bold(), self.config.rag_max_files.to_string().yellow());
        println!("  {}: {} bytes", "RAG Max File Size".bold(), self.config.rag_max_file_size.to_string().yellow());
        println!("  {}: {}", "System Prompt".bold(),
            if self.config.system_prompt.len() > 50 {
                format!("{}...", &self.config.system_prompt[..50])
            } else {
                self.config.system_prompt.clone()
            }.yellow()
        );
        println!("  {}: {}", "Config Path".bold(), self.config_path.display().to_string().dimmed());
        println!();
        Ok(())
    }

    fn get_multiline_input(&self) -> Result<String> {
        println!("\n{}", "Enter your prompt (Ctrl+D or type 'END' on a new line to finish):".dimmed());
        println!("{}", "─".repeat(60).dimmed());
        
        let mut input = String::new();
        let stdin = io::stdin();
        let mut buffer = String::new();
        
        loop {
            buffer.clear();
            match stdin.read_line(&mut buffer) {
                Ok(0) => break, // EOF (Ctrl+D)
                Ok(_) => {
                    if buffer.trim() == "END" {
                        break;
                    }
                    input.push_str(&buffer);
                }
                Err(e) => anyhow::bail!("Error reading input: {}", e),
            }
        }
        
        Ok(input.trim().to_string())
    }

    fn send_prompt(&self, prompt: String, model: Option<String>, temperature: Option<f32>) -> Result<()> {
        let model = model.unwrap_or_else(|| self.config.default_model.clone());
        let temperature = temperature.unwrap_or(self.config.temperature);

        println!("\n{} {} {} {}...",
            "🤖".to_string(),
            "Using".dimmed(),
            model.green().bold(),
            "(thinking)".dimmed()
        );

        // Gather directory context for RAG
        let rag_context = self.gather_directory_context()?;

        // Build system prompt with RAG context
        let system_prompt = if !rag_context.is_empty() {
            format!("{}\n{}", self.config.system_prompt, rag_context)
        } else {
            self.config.system_prompt.clone()
        };

        let request = OllamaRequest {
            model: model.clone(),
            prompt,
            system: Some(system_prompt),
            temperature,
            stream: self.config.stream,
        };

        let url = format!("{}/api/generate", self.config.ollama_host);

        if self.config.stream {
            self.send_streaming_request(url, request)?;
        } else {
            self.send_blocking_request(url, request)?;
        }

        Ok(())
    }

    fn send_streaming_request(&self, url: String, request: OllamaRequest) -> Result<()> {
        let response = self.client
            .post(&url)
            .json(&request)
            .send()?;
        
        if !response.status().is_success() {
            anyhow::bail!("Request failed: {}", response.status());
        }

        println!("\n{}", "─".repeat(60).dimmed());
        print!("{} ", "Assistant:".bold().cyan());
        io::stdout().flush()?;
        
        let reader = std::io::BufReader::new(response);
        use std::io::BufRead;
        
        for line in reader.lines() {
            let line = line?;
            if let Ok(chunk) = serde_json::from_str::<OllamaStreamResponse>(&line) {
                print!("{}", chunk.response);
                io::stdout().flush()?;
                
                if chunk.done {
                    println!("\n{}", "─".repeat(60).dimmed());
                    break;
                }
            }
        }
        
        Ok(())
    }

    fn send_blocking_request(&self, url: String, request: OllamaRequest) -> Result<()> {
        let response = self.client
            .post(&url)
            .json(&request)
            .send()?;
        
        if !response.status().is_success() {
            anyhow::bail!("Request failed: {}", response.status());
        }

        let data: OllamaResponse = response.json()?;
        
        println!("\n{}", "─".repeat(60).dimmed());
        println!("{} {}", "Assistant:".bold().cyan(), data.response);
        println!("{}", "─".repeat(60).dimmed());
        
        Ok(())
    }

    fn set_default_model(&mut self, model: String) -> Result<()> {
        self.config.default_model = model.clone();
        self.save_config()?;
        println!("✅ Default model set to: {}", model.green().bold());
        Ok(())
    }

    fn reset_config(&mut self) -> Result<()> {
        self.config = Config::default();
        self.save_config()?;
        println!("✅ Configuration reset to defaults");
        self.show_config()?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut assistant = Assistant::new()?;

    // Handle configuration commands first
    if let Some(model) = args.set_default {
        return assistant.set_default_model(model);
    }

    if args.list {
        return assistant.list_models();
    }

    if args.config {
        return assistant.show_config();
    }

    if args.reset {
        return assistant.reset_config();
    }

    // Handle prompts
    let prompt = if args.prompt.is_empty() {
        // Interactive mode
        assistant.get_multiline_input()?
    } else {
        // Inline mode
        args.prompt.join(" ")
    };

    if prompt.is_empty() {
        println!("{}", "No prompt provided. Use --help for usage information.".yellow());
        return Ok(());
    }

    assistant.send_prompt(prompt, args.model, args.temperature)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.default_model, "llama3.2");
        assert_eq!(config.ollama_host, "http://host.docker.internal:11434");
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.stream, true);
        assert_eq!(config.enable_rag, true);
        assert_eq!(config.rag_max_files, 50);
        assert_eq!(config.rag_max_file_size, 10240);
        assert!(config.system_prompt.contains("concise"));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let serialized = toml::to_string(&config).expect("Failed to serialize config");
        assert!(serialized.contains("default_model"));
        assert!(serialized.contains("llama3.2"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            default_model = "llama3.2"
            ollama_host = "http://localhost:11434"
            temperature = 0.5
            stream = false
        "#;
        let config: Config = toml::from_str(toml_str).expect("Failed to deserialize config");
        assert_eq!(config.default_model, "llama3.2");
        assert_eq!(config.ollama_host, "http://localhost:11434");
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.stream, false);
    }

    #[test]
    fn test_ollama_request_creation() {
        let request = OllamaRequest {
            model: String::from("llama3.2"),
            prompt: String::from("Hello"),
            system: Some(String::from("You are a helpful assistant")),
            temperature: 0.7,
            stream: true,
        };
        assert_eq!(request.model, "llama3.2");
        assert_eq!(request.prompt, "Hello");
        assert_eq!(request.system, Some(String::from("You are a helpful assistant")));
        assert_eq!(request.temperature, 0.7);
        assert_eq!(request.stream, true);
    }

    #[test]
    fn test_ollama_request_serialization() {
        let request = OllamaRequest {
            model: String::from("llama3.2"),
            prompt: String::from("Test prompt"),
            system: Some(String::from("Be helpful")),
            temperature: 0.8,
            stream: false,
        };
        let json = serde_json::to_string(&request).expect("Failed to serialize request");
        assert!(json.contains("llama3.2"));
        assert!(json.contains("Test prompt"));
        assert!(json.contains("Be helpful"));
    }

    #[test]
    fn test_ollama_response_deserialization() {
        let json = r#"{"response": "Test response"}"#;
        let response: OllamaResponse = serde_json::from_str(json).expect("Failed to deserialize response");
        assert_eq!(response.response, "Test response");
    }

    #[test]
    fn test_ollama_stream_response_deserialization() {
        let json = r#"{"response": "Test", "done": false}"#;
        let response: OllamaStreamResponse = serde_json::from_str(json).expect("Failed to deserialize stream response");
        assert_eq!(response.response, "Test");
        assert_eq!(response.done, false);
    }
}
