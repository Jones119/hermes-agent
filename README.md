
# Hermes Agent - Full Stack Rust Implementation

A fully functional AI assistant agent written in Rust, with a modern web interface and comprehensive tooling.

## Project Status

✅ **Production-Ready Core**
- Agent engine with tool calling capabilities
- LLM integration (OpenAI-compatible)
- Web API backend (Axum)
- WebSocket real-time communication
- Modern web interface
- Comprehensive tool ecosystem

## Features

### Core Agent
- Asynchronous agent runtime
- Tool registration and execution
- Conversation context management
- Memory and skill systems
- Extensible architecture

### Tooling Ecosystem
- **File Tools**: Read and write files
- **Terminal Tools**: Execute shell commands, grep, list directories
- **Web Tools**: HTTP requests, web search

### Interfaces
- **CLI**: Interactive and single-run modes
- **Web**: Modern dark-themed chat interface
- **API**: REST + WebSocket endpoints

## Quick Start

### Prerequisites
- Rust 1.70+
- Cargo

### Installation
```bash
git clone &lt;repo&gt;
cd workspace
cargo build
```

### Running the Web Server
```bash
cargo run --bin hermes-web -- --config config.toml --port 3000 --host 127.0.0.1 --static-dir ./web
```

Then open your browser at `http://127.0.0.1:3000`

### Running the CLI
```bash
# Interactive mode
cargo run --bin hermes -- interactive

# Single run mode
cargo run --bin hermes -- run "Hello, how are you?"
```

### Running Tests
```bash
cargo test
```

## Project Structure

```
/workspace/
├── Cargo.toml                          # Workspace manifest
├── config.toml                          # Default configuration
├── README.md                            # This file
├── PROJECT_PLAN.md                      # Project roadmap
├── crates/
│   ├── hermes-core/                     # Core agent library
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── agent.rs                 # Agent engine
│   │   │   ├── tool.rs                  # Tool system
│   │   │   ├── llm.rs                   # LLM abstraction
│   │   │   ├── config.rs                # Configuration
│   │   │   ├── context.rs               # Context management
│   │   │   ├── memory.rs                # Memory system
│   │   │   ├── prompt.rs                # Prompt building
│   │   │   ├── skill.rs                 # Skill system
│   │   │   └── error.rs                 # Error handling
│   │   └── tests/
│   │       └── unit_tests.rs            # Core tests
│   ├── hermes-tools/                    # Tool library
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── file_tools.rs            # File operations
│   │   │   ├── terminal.rs              # Terminal operations
│   │   │   └── web_tools.rs             # Web operations
│   │   └── tests/
│   │       └── unit_tests.rs            # Tools tests
│   ├── hermes-cli/                      # CLI application
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   └── hermes-web/                      # Web server
│       ├── Cargo.toml
│       └── src/main.rs
└── web/                                 # Frontend
    ├── index.html
    └── app.js
```

## Configuration

The default `config.toml` is set to use a mock LLM provider for testing:

```toml
[llm]
provider = "mock"  # Set to "openai" for real API
model = "gpt-4o"
api_key = ""
base_url = ""
temperature = 0.7
max_tokens = 4096

[agent]
max_iterations = 20
max_tool_calls = 50
timeout_secs = 600
memory_limit = 100

[web]
host = "127.0.0.1"
port = 3000
enable_cors = true
static_dir = "./web/dist"
```

## API Documentation

### REST API

#### Create Session
```http
POST /sessions
Content-Type: application/json
{ "user_id": "optional" }

Response: { "session_id": "uuid" }
```

#### Send Message
```http
POST /sessions/{session_id}/chat
Content-Type: application/json
{ "input": "Hello!" }

Response: { "output": "Response..." }
```

#### List Sessions
```http
GET /sessions
Response: ["session_id1", "session_id2"]
```

#### Delete Session
```http
DELETE /sessions/{session_id}
Response: 200 OK
```

### WebSocket API

Connect at `ws://localhost:3000/ws`

**Client &rarr; Server:**
```json
{
  "msg_type": "create_session|chat",
  "content": "message",
  "session_id": "uuid"
}
```

**Server &rarr; Client:**
```json
{
  "msg_type": "session_created|thinking|response|error",
  "content": "message",
  "session_id": "uuid"
}
```

## Development

### Adding New Tools
1. Create a new tool struct in `hermes-tools/src/`
2. Implement the `Tool` trait (from `hermes-core::tool`)
3. Register the tool in your agent instance
4. Add tests in `hermes-tools/tests/`

### Building for Release
```bash
cargo build --release
```

## Testing

Run all tests:
```bash
cargo test
```

Test coverage:
```bash
# Install grcov
cargo install grcov
# Run tests with coverage
cargo test -- --test-threads=1 --coverage
# Generate report
grcov ./target/debug/ -s . -t html --llvm --branch --ignore-not-existing -o ./target/debug/coverage/
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with Rust and Tokio
- Web framework: Axum
- Async runtime: Tokio
