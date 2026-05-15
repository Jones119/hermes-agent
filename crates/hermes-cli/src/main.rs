
use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;
use hermes_core::agent::Agent;
use hermes_core::config::Config;
use hermes_core::llm::create_client;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
enum Cli {
    /// Start the agent in interactive mode
    Interactive {
        #[arg(long, short = 'c', default_value = "config.toml")]
        config: String,
    },
    /// Run a single command
    Run {
        #[arg(long, short = 'c', default_value = "config.toml")]
        config: String,
        #[arg(required = true, last = true)]
        prompt: Vec<String>,
    },
}

fn setup_tracing() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing()?;
    
    match Cli::parse() {
        Cli::Interactive { config } => {
            interactive(config).await?;
        }
        Cli::Run { config, prompt } => {
            run_once(config, &prompt.join(" ")).await?;
        }
    }
    
    Ok(())
}

async fn interactive(config_path: String) -> Result<()> {
    let config = Config::load(&config_path).unwrap_or_else(|_| Config::default());
    let llm = create_client(
        &config.llm.provider,
        config.llm.api_key.clone(),
        config.llm.base_url.clone(),
        config.llm.model.clone(),
    );
    
    let mut agent = Agent::new(config, llm);
    agent.register_default_tools().await;
    
    println!("Hermes Agent - Interactive Mode");
    println!("Type 'exit' or 'quit' to end the session\n");
    
    use std::io::{self, BufRead};
    
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = line?;
        
        if input.trim().is_empty() {
            continue;
        }
        
        if input.trim().eq_ignore_ascii_case("exit") || 
           input.trim().eq_ignore_ascii_case("quit") {
            break;
        }
        
        match agent.run(&input).await {
            Ok(output) => println!("\n{}\n", output),
            Err(e) => eprintln!("\nError: {}\n", e),
        }
    }
    
    Ok(())
}

async fn run_once(config_path: String, prompt: &str) -> Result<()> {
    let config = Config::load(&config_path).unwrap_or_else(|_| Config::default());
    let llm = create_client(
        &config.llm.provider,
        config.llm.api_key.clone(),
        config.llm.base_url.clone(),
        config.llm.model.clone(),
    );
    
    let mut agent = Agent::new(config, llm);
    agent.register_default_tools().await;
    
    match agent.run(prompt).await {
        Ok(output) => println!("{}", output),
        Err(e) => eprintln!("Error: {}", e),
    }
    
    Ok(())
}
