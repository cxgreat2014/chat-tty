mod client;
mod errors;
mod state;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use state::AppState;

struct State {
    api_key: String,
}
const API_KEY: &str = "OPENAI_KEY";

#[tokio::main]
async fn main() -> Result<()> {
    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    let key = std::env::var(API_KEY).ok();

    let mut state = AppState::new(key.as_deref());

    while !state.is_ready() {
        println!("Please enter your OpenAI API key:");
        match rl.readline(">> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap_or_else(|e| {
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
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap_or_else(|e| {
                    println!("{:?}", e);
                    false
                });
                // println!("Line: {}", line);
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
    rl.save_history("history.txt").unwrap_or_else(|e| {
        println!("{:?}", e);
    });
    Ok(())
}
