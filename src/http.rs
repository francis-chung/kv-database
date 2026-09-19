use std::sync::Arc;
use axum::{
    extract::State,
    response::IntoResponse,
    routing::post,
    Json,
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{engine::Engine, server::execute_command};

// HTTP API request body for commands that take parameters
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CommandRequest {
    Set {
        key: String,
        value: String,
    },
    Zadd {
        key: String,
        member: String,
        score: f64,
    },
    Zrem {
        key: String,
        member: String,
    },
    Zrange {
        key: String,
        from: isize,
        to: isize,
        with_scores: Option<bool>,
    },
    // for commands that take a single key (GET, DEL, EXISTS, ZSCORE)
    KeyOnly {
        key: String,
    },
}

// HTTP API response body
#[derive(Debug, Serialize)]
struct CommandResponse {
    result: String,
}

// HTTP API error response
// not currently being used
#[derive(Debug, Serialize)]
struct ApiError {
    error: String,
}

#[derive(Clone)]
struct ApiState {
    engine: Arc<Mutex<Engine<tokio::fs::File>>>
}

pub type SharedEngine = Arc<Mutex<Engine<tokio::fs::File>>>;

// converts HTTP API requests to server's wire format 
// and calls shared execute_command function
async fn handle_command(
    State(state): State<ApiState>,
    Json(request): Json<CommandRequest>,
) -> impl IntoResponse {
    let command = match request {
        CommandRequest::KeyOnly { key } => key,
        CommandRequest::Set { key, value } => format!("SET {key} {value}"),
        CommandRequest::Zadd { key, member, score } => format!("ZADD {key} {member} {score}"),
        CommandRequest::Zrem { key, member } => format!("ZREM {key} {member}"),
        CommandRequest::Zrange { key, from, to, with_scores } => {
            let mut cmd = format!("ZRANGE {key} {from} {to}");
            if with_scores.unwrap_or(false) {
                cmd.push_str(" WITHSCORES");
            }
            cmd
        }
    };

    let response = execute_command(&state.engine, &command).await;
    Json(CommandResponse { result: response })
}

pub fn create_router(engine: SharedEngine) -> Router {
    Router::new()
        .route("/api/command", post(handle_command))
        .with_state(ApiState { engine })
}

pub async fn start_http_server(
    engine: SharedEngine,
    addr: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let router = create_router(engine);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}