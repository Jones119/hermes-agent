
# Hermes Agent (Rust Rewrite)

This is a full-stack Rust rewrite of the Hermes Agent with a modern Web UI.

## Project Structure

```
/workspace/
├── Cargo.toml                      # Workspace configuration
├── config.toml                     # Default configuration
├── PROJECT_PLAN.md                 # Project tracking
├── README_RUST.md                  # This file
├── crates/
│   ├── hermes-core/               # Core agent library
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── agent.rs          # Agent core
│   │   │   ├── config.rs         # Configuration
│   │   │   ├── context.rs        # Conversation context
│   │   │   ├── error.rs          # Error types
│   │   │   ├── llm.rs            # LLM client abstraction
│   │   │   ├── memory.rs         # Memory store
│   │   │   ├── prompt.rs         # Prompt building
│   │   │   ├── skill.rs          # Skill system
│   │   │   └── tool.rs           # Tool system
│   │   └── Cargo.toml
│   ├── hermes-tools/             # Tools library
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── file_tools.rs
│   │   └── Cargo.toml
│   ├── hermes-cli/               # CLI application
│   │   ├── src/main.rs
│   │   └── Cargo.toml
│   └── hermes-web/               # Web server
│       ├── src/main.rs
│       └── Cargo.toml
└── web/                           # Web frontend
    ├── index.html
    └── app.js
```

## Getting Started

### Prerequisites

- Rust 1.70+
- Cargo

### Build

```bash
cargo build
```

### Configure

Edit `config.toml` to set your LLM API key:

```toml
[llm]
provider = "openai"
model = "gpt-4o"
api_key = "your-api-key-here"
```

### Run

#### Interactive Mode

```bash
cargo run --bin hermes -- interactive
```

#### Run Single Command

```bash
cargo run --bin hermes -- run "Hello, world!"
```

#### Web Server

```bash
# Build web server
cargo build --bin hermes-web

# Run
cargo run --bin hermes-web
```

Then open `web/index.html` in your browser.

## Architecture

### hermes-core

The core library contains:
- `Agent`: Main agent orchestrator
- `ToolRegistry`: Tool management
- `LlmClient`: LLM provider abstraction
- `ConversationContext`: Context management
- `MemoryStore`: Memory storage

### hermes-tools

Tool implementations:
- `ReadFileTool`: Read files
- `WriteFileTool`: Write files
- (more to come)

### hermes-web

Axum web server providing:
- REST API for agent interaction
- Session management
- Static file serving

### hermes-cli

Command-line interface with:
- Interactive mode
- Single-shot mode
- Web server launcher

## Roadmap

- [x] Project structure
- [x] Core agent logic
- [x] Tool system
- [x] LLM abstraction
- [x] Basic CLI
- [x] Web API
- [x] Simple Web UI
- [ ] More tools (terminal, web, etc.)
- [ ] Skill system
- [ ] Memory persistence
- [ ] Streaming responses
- [ ] WebSocket support
- [ ] Enhanced UI

## Development

### Add a new tool

1. Implement the `Tool` trait in `hermes-tools`
2. Register it in the agent setup
3. Test and iterate

### Add a new LLM provider

1. Implement the `LlmClient` trait
2. Add to `create_client()` factory
3. Update configuration

## License

MIT
