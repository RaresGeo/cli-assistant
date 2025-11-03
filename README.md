# Assistant CLI

A Rust-based CLI tool for interacting with Ollama AI models from WSL, designed for seamless AI-powered command-line assistance.

## Features

- 🚀 **Quick inline commands**: `assistant show all remotes for this git project`
- 📝 **Interactive multiline mode**: Just type `assistant` and enter longer prompts
- ⚙️ **Configurable defaults**: Set your preferred model and settings
- 🎯 **Model override**: Use different models on-the-fly with `-m` flag
- 🌊 **Streaming responses**: Real-time output as the model generates
- 🎨 **Colored output**: Clear, readable terminal interface

## Installation

### Prerequisites

1. **Ollama running on Windows host**
   - Ensure Ollama is installed and running on your Windows system
   - Default port should be 11434

2. **Rust toolchain in WSL**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

### Build and Install

```bash
# Clone or navigate to the project directory
cd assistant-cli

# Build the release version
cargo build --release

# Install to your local bin directory
cargo install --path .

# Or copy directly to /usr/local/bin for system-wide access
sudo cp target/release/assistant /usr/local/bin/
```

## Usage

### Basic Commands

```bash
# Inline mode - quick questions
assistant what is the current directory structure

# Interactive mode - longer prompts
assistant
# Then type your multi-line prompt, end with Ctrl+D or 'END'

# Use a different model
assistant -m codellama:13b write a Python function to calculate fibonacci

# With custom temperature
assistant -t 0.9 write a creative story about robots
```

### Configuration Management

```bash
# Show current configuration
assistant --config

# List available models
assistant --list

# Set default model
assistant --set-default llama3.2

# Reset to default configuration
assistant --reset
```

### Examples

```bash
# Git operations
assistant show all remotes for this git project
assistant explain the last 5 commits

# Code assistance
assistant -m codellama explain this error: "undefined reference to main"

# System operations
assistant how to find large files in current directory

# General questions with specific model
assistant -m mixtral:8x7b what are the best practices for rust error handling
```

## Configuration

Configuration file is stored at: `~/.config/assistant-cli/config.toml`

```toml
default_model = "llama3.2"
ollama_host = "http://host.docker.internal:11434"
temperature = 0.7
stream = true
```

### Configuration Options

- **default_model**: The model to use unless overridden
- **ollama_host**: URL to reach Ollama from WSL
  - `host.docker.internal:11434` - Works in most WSL2 setups
  - You can also use your Windows IP directly
- **temperature**: Creativity level (0.0 = deterministic, 1.0 = creative)
- **stream**: Enable real-time streaming of responses

## Network Configuration

If `host.docker.internal` doesn't work, you can find your Windows IP:

1. In Windows PowerShell:
   ```powershell
   ipconfig
   ```
   Look for the WSL adapter IP address

2. Or from WSL:
   ```bash
   ip route | grep default | awk '{print $3}'
   ```

Then update your config:
```toml
ollama_host = "http://YOUR_WINDOWS_IP:11434"
```

## Tips

1. **Create an alias** for even quicker access:
   ```bash
   echo 'alias ai="assistant"' >> ~/.bashrc
   source ~/.bashrc
   # Now use: ai what is the time
   ```

2. **Model selection**: 
   - Use lighter models (like `llama3.2`) for quick tasks
   - Use larger models (like `mixtral`) for complex reasoning
   - Use code-specific models (like `codellama`) for programming

3. **Multiline mode tips**:
   - Great for pasting code snippets
   - Use `END` on a new line as an alternative to Ctrl+D
   - Supports markdown formatting in prompts

## Troubleshooting

### Connection Issues
- Ensure Ollama is running on Windows: `ollama serve`
- Check Windows Firewall isn't blocking port 11434
- Try using your Windows IP instead of `host.docker.internal`

### Model Not Found
- List available models: `assistant --list`
- Pull new models in Windows: `ollama pull model-name`

### Performance
- Streaming mode provides better UX for long responses
- Adjust temperature based on task (lower for factual, higher for creative)

## License

MIT
