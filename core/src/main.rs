mod ipc;

use std::error::Error;
use std::io::{self, BufRead};
use ipc::models::NeutralinoConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Orbit Core Engine starting up...");

    // Neutralinojs sends config details over stdin as JSON
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut input = String::new();

    // Read the first line which contains the config
    if reader.read_line(&mut input)? == 0 {
        return Err("No config received on stdin".into());
    }

    let config: NeutralinoConfig = serde_json::from_str(&input.trim())?;
    println!("Configuration parsed. Connecting to port: {}", config.nl_port);

    let ws_stream = ipc::connect(&config).await?;
    ipc::run_loop(ws_stream).await?;

    Ok(())
}
