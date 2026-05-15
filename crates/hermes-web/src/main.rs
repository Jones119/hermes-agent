
use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::{error, info, warn};
use uuid::Uuid;
use hermes_core::agent::Agent;
use hermes_core::config::Config;
use hermes_core::context::AgentContext;
use hermes_core::error::Error as HermesError;
use hermes_core::llm::create_client;


#[derive(Parser)]
struct Cli {
    #[arg(long, short = 'c', default_value = "config.toml")]
    config: String,

    #[arg(long, short = 'p', default_value = "3000")]
    port: u16,

    #[arg(long, short = 'H', default_value = "127.0.0.1")]
    host: String,

    #[arg(long, short = 's', default_value = "./web")]
    static_dir: String,
}

struct AppState {
    agents: RwLock<HashMap<String, Arc<RwLock<Agent>>>>,
    config: Config,
}

#[derive(Deserialize)]
struct CreateSessionRequest {
    user_id: Option<String>,
}

#[derive(Serialize)]
struct CreateSessionResponse {
    session_id: String,
}

#[derive(Deserialize)]
struct ChatRequest {
    input: String,
}

#[derive(Serialize)]
struct ChatResponse {
    output: String,
}

#[derive(Serialize, Deserialize)]
struct WsMessage {
    msg_type: String,
    content: String,
    session_id: Option<String>,
}

fn error_to_response(e: HermesError) -> (StatusCode, String) {
    let status = match &e {
        HermesError::ToolNotFound(_) => StatusCode::NOT_FOUND,
        HermesError::InvalidArguments(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (status, e.to_string())
}

async fn health_check() -> &'static str {
    "ok"
}

async fn get_config(State(state): State<Arc<AppState>>) -> Json<Config> {
    Json(state.config.clone())
}

#[derive(Serialize)]
struct ToolResponse {
    name: String,
    description: String,
    parameters: Vec<ToolParameterResponse>,
}

#[derive(Serialize)]
struct ToolParameterResponse {
    name: String,
    description: String,
}

fn sanitize_input(input: &str) -> Result<String, (StatusCode, String)> {
    let dangerous_patterns = [
        ";", "&&", "||", "`", "$(", "(", ")", "{", "}", "[", "]",
        "<", ">", "|", "\\", "/", "..", "\0"
    ];

    for pattern in dangerous_patterns.iter() {
        if input.contains(pattern) {
            return Err((StatusCode::BAD_REQUEST, "Input contains dangerous characters".into()));
        }
    }

    Ok(input.to_string())
}

fn validate_path(path: &str) -> Result<String, (StatusCode, String)> {
    if path.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Path cannot be empty".into()));
    }

    if path.contains("..") {
        return Err((StatusCode::BAD_REQUEST, "Path traversal not allowed".into()));
    }

    if path.starts_with("/") || path.starts_with("\\") {
        return Err((StatusCode::BAD_REQUEST, "Absolute paths not allowed".into()));
    }

    Ok(path.to_string())
}

fn is_command_allowed(command: &str, config: &Config) -> bool {
    if config.security.command_whitelist.is_empty() {
        return true;
    }

    let cmd = command.split_whitespace().next().unwrap_or("");
    config.security.command_whitelist.contains(&cmd.to_string())
}

async fn get_tools() -> Json<Vec<ToolResponse>> {
    let tools = vec![
        ToolResponse {
            name: "read_file".to_string(),
            description: "Read the contents of a file".to_string(),
            parameters: vec![
                ToolParameterResponse {
                    name: "path".to_string(),
                    description: "The path to the file".to_string(),
                },
            ],
        },
        ToolResponse {
            name: "write_file".to_string(),
            description: "Write content to a file".to_string(),
            parameters: vec![
                ToolParameterResponse {
                    name: "path".to_string(),
                    description: "The path to the file".to_string(),
                },
                ToolParameterResponse {
                    name: "content".to_string(),
                    description: "The content to write".to_string(),
                },
            ],
        },
        ToolResponse {
            name: "terminal".to_string(),
            description: "Execute a shell command".to_string(),
            parameters: vec![
                ToolParameterResponse {
                    name: "command".to_string(),
                    description: "The command to execute".to_string(),
                },
            ],
        },
        ToolResponse {
            name: "grep".to_string(),
            description: "Search for text in files".to_string(),
            parameters: vec![
                ToolParameterResponse {
                    name: "pattern".to_string(),
                    description: "The search pattern".to_string(),
                },
                ToolParameterResponse {
                    name: "path".to_string(),
                    description: "The path to search".to_string(),
                },
            ],
        },
        ToolResponse {
            name: "list_dir".to_string(),
            description: "List directory contents".to_string(),
            parameters: vec![
                ToolParameterResponse {
                    name: "path".to_string(),
                    description: "The directory path".to_string(),
                },
            ],
        },
        ToolResponse {
            name: "http_get".to_string(),
            description: "Make an HTTP GET request".to_string(),
            parameters: vec![
                ToolParameterResponse {
                    name: "url".to_string(),
                    description: "The URL to fetch".to_string(),
                },
            ],
        },
        ToolResponse {
            name: "http_post".to_string(),
            description: "Make an HTTP POST request".to_string(),
            parameters: vec![
                ToolParameterResponse {
                    name: "url".to_string(),
                    description: "The URL to post to".to_string(),
                },
                ToolParameterResponse {
                    name: "body".to_string(),
                    description: "The request body".to_string(),
                },
            ],
        },
    ];
    Json(tools)
}

async fn create_session(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, (StatusCode, String)> {
    let session_id = Uuid::new_v4().to_string();

    let config = state.config.clone();
    let llm = create_client(
        &config.llm.provider,
        config.llm.api_key.clone(),
        config.llm.base_url.clone(),
        config.llm.model.clone(),
    );

    let mut context = AgentContext::new().with_session_id(session_id.clone());
    if let Some(user_id) = req.user_id {
        context = context.with_user_id(user_id);
    }

    let mut agent = Agent::new(config, llm).with_context(context);
    agent.register_default_tools().await;

    state.agents.write().await.insert(
        session_id.clone(),
        Arc::new(RwLock::new(agent)),
    );

    info!("Created new session: {}", session_id);

    Ok(Json(CreateSessionResponse { session_id }))
}

async fn chat(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, String)> {
    let sanitized_input = sanitize_input(&req.input)?;

    if sanitized_input.len() > 4096 {
        return Err((StatusCode::BAD_REQUEST, "Input too long".into()));
    }

    let agent_arc = {
        let agents = state.agents.read().await;
        agents
            .get(&session_id)
            .cloned()
            .ok_or((StatusCode::NOT_FOUND, "Session not found".into()))?
    };

    let mut agent = agent_arc.write().await;
    let output = agent
        .run(&sanitized_input)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ChatResponse { output }))
}

async fn list_sessions(State(state): State<Arc<AppState>>) -> Json<Vec<String>> {
    let agents = state.agents.read().await;
    Json(agents.keys().cloned().collect())
}

async fn delete_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> StatusCode {
    let mut agents = state.agents.write().await;
    if agents.remove(&session_id).is_some() {
        info!("Deleted session: {}", session_id);
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(
    ws: WebSocket,
    state: Arc<AppState>,
) {
    let (mut sender, mut receiver) = ws.split();
    let mut current_session_id: Option<String> = None;

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(ws_msg) => {
                        match ws_msg.msg_type.as_str() {
                            "create_session" => {
                                let session_id = Uuid::new_v4().to_string();
                                let config = state.config.clone();
                                let llm = create_client(
                                    &config.llm.provider,
                                    config.llm.api_key.clone(),
                                    config.llm.base_url.clone(),
                                    config.llm.model.clone(),
                                );

                                let context =
                                    AgentContext::new().with_session_id(session_id.clone());

                                let mut agent = Agent::new(config, llm).with_context(context);
                                agent.register_default_tools().await;

                                state.agents.write().await.insert(
                                    session_id.clone(),
                                    Arc::new(RwLock::new(agent)),
                                );

                                current_session_id = Some(session_id.clone());

                                let _ = sender.send(Message::Text(serde_json::to_string(
                                    &WsMessage {
                                        msg_type: "session_created".into(),
                                        content: session_id,
                                        session_id: None,
                                    },
                                ).unwrap())).await;
                            }
                            "chat" => {
                                if let Some(ref session_id) = current_session_id {
                                    let agent_arc = {
                                        let agents = state.agents.read().await;
                                        agents.get(session_id).cloned()
                                    };

                                    if let Some(agent_arc) = agent_arc {
                                        let _ = sender.send(Message::Text(serde_json::to_string(
                                            &WsMessage {
                                                msg_type: "thinking".into(),
                                                content: "Processing...".into(),
                                                session_id: None,
                                            },
                                        ).unwrap())).await;

                                        let input = ws_msg.content.clone();
                                        let mut agent = agent_arc.write().await;

                                        match agent.run(&input).await {
                                            Ok(output) => {
                                                let _ = sender.send(Message::Text(
                                                    serde_json::to_string(&WsMessage {
                                                        msg_type: "response".into(),
                                                        content: output,
                                                        session_id: None,
                                                    }).unwrap(),
                                                )).await;
                                            }
                                            Err(e) => {
                                                let _ = sender.send(Message::Text(
                                                    serde_json::to_string(&WsMessage {
                                                        msg_type: "error".into(),
                                                        content: e.to_string(),
                                                        session_id: None,
                                                    }).unwrap(),
                                                )).await;
                                            }
                                        }
                                    } else {
                                        let _ = sender.send(Message::Text(serde_json::to_string(
                                            &WsMessage {
                                                msg_type: "error".into(),
                                                content: "No active session".into(),
                                                session_id: None,
                                            },
                                        ).unwrap())).await;
                                    }
                                } else {
                                    let _ = sender.send(Message::Text(serde_json::to_string(
                                        &WsMessage {
                                            msg_type: "error".into(),
                                            content: "No session created".into(),
                                            session_id: None,
                                        },
                                    ).unwrap())).await;
                                }
                            }
                            _ => {
                                warn!("Unknown message type: {}", ws_msg.msg_type);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse WebSocket message: {}", e);
                        let _ = sender.send(Message::Text(serde_json::to_string(&WsMessage {
                            msg_type: "error".into(),
                            content: format!("Invalid message format: {}", e),
                            session_id: None,
                        }).unwrap())).await;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket closed");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    info!("WebSocket handler finished");
}

fn router(state: Arc<AppState>, static_dir: PathBuf) -> Router {
    let static_service = ServeDir::new(&static_dir);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_check))
        .route("/config", get(get_config))
        .route("/tools", get(get_tools))
        .route("/sessions", post(create_session))
        .route("/sessions", get(list_sessions))
        .route("/sessions/:session_id", delete(delete_session))
        .route("/sessions/:session_id/chat", post(chat))
        .route("/ws", get(websocket_handler))
        .nest_service("/", static_service)
        .layer(cors)
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    let cli = Cli::parse();
    println!("Loading config from: {}", cli.config);
    let config = Config::load(&cli.config).unwrap_or_else(|e| {
        println!("Config load error: {}, using default", e);
        Config::default()
    });

    println!("Loaded LLM provider: {}", config.llm.provider);
    println!("Loaded LLM model: {}", config.llm.model);

    let addr = format!("{}:{}", cli.host, cli.port);
    let static_dir = PathBuf::from(&cli.static_dir);

    info!("Static files will be served from: {:?}", static_dir);

    let state = Arc::new(AppState {
        agents: RwLock::new(HashMap::new()),
        config: config.clone(),
    });

    let app = router(state, static_dir);

    info!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
