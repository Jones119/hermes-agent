
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
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
    
    #[arg(long, short = 'h', default_value = "127.0.0.1")]
    host: String,
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

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn error_to_response(e: HermesError) -> (StatusCode, Json<ErrorResponse>) {
    let status = match &e {
        HermesError::ToolNotFound(_) => StatusCode::NOT_FOUND,
        HermesError::InvalidArguments(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    
    (status, Json(ErrorResponse {
        error: e.to_string(),
    }))
}

async fn health_check() -> &'static str {
    "ok"
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
    
    let mut context = AgentContext::new()
        .with_session_id(session_id.clone());
    if let Some(user_id) = req.user_id {
        context = context.with_user_id(user_id);
    }
    
    let mut agent = Agent::new(config, llm)
        .with_context(context);
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
    let agent_arc = {
        let agents = state.agents.read().await;
        agents.get(&session_id)
            .cloned()
            .ok_or((StatusCode::NOT_FOUND, "Session not found".into()))?
    };
    
    let mut agent = agent_arc.write().await;
    let output = agent.run(&req.input).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(ChatResponse { output }))
}

async fn list_sessions(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<String>> {
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

fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/sessions", post(create_session))
        .route("/sessions", get(list_sessions))
        .route("/sessions/:session_id", delete(delete_session))
        .route("/sessions/:session_id/chat", post(chat))
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();
    
    let cli = Cli::parse();
    let config = Config::load(&cli.config)
        .unwrap_or_else(|_| Config::default());
    
    let addr = format!("{}:{}", cli.host, cli.port);
    
    let state = Arc::new(AppState {
        agents: RwLock::new(HashMap::new()),
        config: config.clone(),
    });
    
    let app = router(state);
    
    info!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
