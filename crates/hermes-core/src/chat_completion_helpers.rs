
use crate::error::{Error, Result};
use crate::llm::{ChatRequest, ChatResponse, LlmClient};
use std::time::Duration;
use tracing::{info, warn};
use tokio::time::sleep;

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_SECS: u64 = 1;
const MAX_BACKOFF_SECS: u64 = 30;

pub struct ChatCompletionConfig {
    pub max_retries: u32,
    pub initial_backoff_secs: u64,
    pub max_backoff_secs: u64,
}

impl Default for ChatCompletionConfig {
    fn default() -> Self {
        ChatCompletionConfig {
            max_retries: MAX_RETRIES,
            initial_backoff_secs: INITIAL_BACKOFF_SECS,
            max_backoff_secs: MAX_BACKOFF_SECS,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorClass {
    Retryable,
    RateLimit,
    ContextLimit,
    Fatal,
}

pub fn classify_api_error(status: u16, error_body: &str) -> ErrorClass {
    match status {
        429 => ErrorClass::RateLimit,
        400 => {
            if error_body.contains("context_length_exceeded")
                || error_body.contains("max_tokens")
                || error_body.contains("too many tokens")
            {
                ErrorClass::ContextLimit
            } else {
                ErrorClass::Fatal
            }
        }
        401 | 403 => ErrorClass::Fatal,
        404 => ErrorClass::Fatal,
        500 | 502 | 503 | 504 => ErrorClass::Retryable,
        _ => ErrorClass::Retryable,
    }
}

pub async fn chat_with_retry(
    client: &dyn LlmClient,
    request: ChatRequest,
    config: &ChatCompletionConfig,
) -> Result<ChatResponse> {
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        if attempt > 0 {
            let backoff = calculate_backoff(attempt, config);
            info!(
                "Retry attempt {}/{} after {}ms",
                attempt, config.max_retries, backoff.as_millis()
            );
            sleep(backoff).await;
        }

        match client.chat(request.clone()).await {
            Ok(response) => return Ok(response),
            Err(Error::Llm(msg)) => {
                let (status, body) = parse_error_details(&msg);
                let error_class = classify_api_error(status, &body);

                match error_class {
                    ErrorClass::Fatal => {
                        warn!("Fatal API error (status {}): {}", status, body);
                        return Err(Error::Llm(msg));
                    }
                    ErrorClass::ContextLimit => {
                        warn!("Context limit exceeded: {}", body);
                        return Err(Error::Llm(msg));
                    }
                    ErrorClass::RateLimit => {
                        let retry_after = parse_retry_after(&body);
                        warn!(
                            "Rate limited, waiting {}ms before retry",
                            retry_after.as_millis()
                        );
                        sleep(retry_after).await;
                        last_error = Some(Error::Llm(msg));
                    }
                    ErrorClass::Retryable => {
                        warn!("Retryable error (status {}): {}", status, body);
                        last_error = Some(Error::Llm(msg));
                    }
                }
            }
            Err(e) => {
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| Error::Llm("Max retries exceeded".into())))
}

fn calculate_backoff(attempt: u32, config: &ChatCompletionConfig) -> Duration {
    let base = config.initial_backoff_secs;
    let max = config.max_backoff_secs;
    let exponential = base.saturating_mul(2u64.saturating_pow(attempt.saturating_sub(1)));
    let jittered = exponential.saturating_add(
        (rand_factor() * exponential as f64 * 0.1) as u64
    );
    Duration::from_millis(jittered.saturating_mul(1000).min(max * 1000))
}

fn rand_factor() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    nanos as f64 / u32::MAX as f64
}

fn parse_error_details(error_msg: &str) -> (u16, String) {
    if let Some(start) = error_msg.find("API error: ") {
        let rest = &error_msg[start + 11..];
        if let Some(space_pos) = rest.find(" - ") {
            if let Ok(status) = rest[..space_pos].parse::<u16>() {
                return (status, rest[space_pos + 3..].to_string());
            }
        }
    }
    (0, error_msg.to_string())
}

fn parse_retry_after(_body: &str) -> Duration {
    Duration::from_secs(5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_api_error() {
        assert_eq!(classify_api_error(429, ""), ErrorClass::RateLimit);
        assert_eq!(classify_api_error(401, ""), ErrorClass::Fatal);
        assert_eq!(classify_api_error(403, ""), ErrorClass::Fatal);
        assert_eq!(classify_api_error(500, ""), ErrorClass::Retryable);
        assert_eq!(classify_api_error(502, ""), ErrorClass::Retryable);
        assert_eq!(
            classify_api_error(400, "context_length_exceeded"),
            ErrorClass::ContextLimit
        );
        assert_eq!(classify_api_error(400, "invalid request"), ErrorClass::Fatal);
    }

    #[test]
    fn test_parse_error_details() {
        let (status, body) = parse_error_details("API error: 429 - rate limited");
        assert_eq!(status, 429);
        assert_eq!(body, "rate limited");
    }
}
