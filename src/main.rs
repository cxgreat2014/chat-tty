mod client;
mod errors;
mod state;
mod storage;

use rustyline::error::ReadlineError;
use rustyline::Result;
use state::AppState;
use storage::Storage;

const BASE: &str = "OPENAI_API_BASE";
const API_KEY: &str = "OPENAI_API_KEY";

#[tokio::main]
async fn main() -> Result<()> {
    let mut storage = Storage::new();

    storage.load_history();

    let base = std::env::var(BASE).ok();
    let key = std::env::var(API_KEY).ok();

    let mut state = AppState::new(base.as_deref(), key.as_deref());

    while !state.is_ready() {
        println!("Please enter your OpenAI API key:");
        match storage.rl.readline(">> ") {
            Ok(line) => {
                storage
                    .rl
                    .add_history_entry(line.as_str())
                    .unwrap_or_else(|e| {
                        println!("{:?}", e);
                        false
                    });
                state.set_api_key(line.as_str());
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    while state.is_ready() {
        let readline = storage.rl.readline(">> ");
        match readline {
            Ok(line) => {
                storage
                    .rl
                    .add_history_entry(line.as_str())
                    .unwrap_or_else(|e| {
                        println!("{:?}", e);
                        false
                    });
                state.add_message(line).await.unwrap_or_else(|e| {
                    println!("{:?}", e);
                });
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    storage.write_history();

    Ok(())
}
