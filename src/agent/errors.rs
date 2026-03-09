use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("HTTP request failed")]
    Request(#[from] reqwest::Error),

    #[error("JSON parsing failed")]
    Json(#[from] serde_json::Error),
}
