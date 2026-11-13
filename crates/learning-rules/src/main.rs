use learning_rules::{calculate, persistence, Error, Request, Result};
use serde_json::json;
use std::io::{self, Read};

fn run() -> Result<serde_json::Value> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        return Err(Error::new(
            "usage",
            "learning-rules <command> < request.json",
        ));
    }
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let request: Request = serde_json::from_str(&input)?;
    match args[0].as_str() {
        "save-attempt"
        | "save-observation"
        | "save-strategy-application"
        | "publish"
        | "audit-learner-model"
        | "inspect-state" => persistence::dispatch(&args[0], request),
        _ => calculate(&args[0], request),
    }
}
fn main() {
    match run() {
        Ok(value) => println!("{value}"),
        Err(error) => {
            println!("{}", json!({"schema_version":1,"error":error}));
            std::process::exit(1);
        }
    }
}
