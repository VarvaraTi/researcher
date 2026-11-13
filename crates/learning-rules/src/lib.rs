//! Learning semantics only. OKF owns storage, references, transactions and recovery.
pub mod learner_model;
pub mod persistence;
pub mod rules;
pub mod schedule;
pub mod studies;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const VERSION: u32 = 1;
pub const RULE_VERSION: &str = "foundation-v1";
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub struct Error {
    pub code: String,
    pub message: String,
}
impl Error {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::new("io", e.to_string())
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::new("json", e.to_string())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: u32,
    pub data: Value,
}

pub fn calculate(command: &str, request: Request) -> Result<Value> {
    if request.schema_version != VERSION {
        return Err(Error::new(
            "schema_version",
            "Only schema_version 1 is supported",
        ));
    }
    let value = match command {
        "check-prerequisites" => rules::check_prerequisites(serde_json::from_value(request.data)?)?,
        "resolve-assessments" => {
            rules::resolve_assessments(&serde_json::from_value(request.data)?)?
        }
        "resolve-settings" => rules::resolve_settings(serde_json::from_value(request.data)?)?,
        "grade-answer" => rules::grade_answer(serde_json::from_value(request.data)?)?,
        "compute-progress" => rules::compute_progress(serde_json::from_value(request.data)?)?,
        "review-support" => rules::review_support(serde_json::from_value(request.data)?)?,
        "check-learning-state" => rules::check_state(&serde_json::from_value(request.data)?)?,
        "compute-schedule" => schedule::compute(serde_json::from_value(request.data)?)?,
        _ => {
            return Err(Error::new(
                "command",
                format!("Unknown calculation: {command}"),
            ))
        }
    };
    Ok(
        serde_json::json!({"schema_version": VERSION, "rule_version": RULE_VERSION, "result": value}),
    )
}
