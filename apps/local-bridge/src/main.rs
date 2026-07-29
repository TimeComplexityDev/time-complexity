use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 9001)]
    port: u16,

    #[arg(long)]
    pair_token: Option<String>,

    #[arg(long)]
    input_file: Option<String>,

    #[arg(long)]
    reset_pairing: bool,
}

#[derive(Serialize, Deserialize)]
struct Config {
    pair_token: String,
}

fn config_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|home| home.join(".config").join("timebridge"))
        .context("$HOME not set; cannot determine config directory")
}

fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

fn load_config() -> Result<Option<Config>> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(&path).context("failed to read config file")?;
    let config: Config = serde_json::from_str(&contents).context("failed to parse config file")?;
    Ok(Some(config))
}

fn save_config(config: &Config) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir).context("failed to create config directory")?;
    let path = config_path()?;
    let json = serde_json::to_string_pretty(config).context("failed to serialize config")?;
    fs::write(&path, json).context("failed to write config file")?;
    Ok(())
}

fn reset_pairing() -> Result<()> {
    let path = config_path()?;
    if path.exists() {
        fs::remove_file(&path).context("failed to remove config file")?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.reset_pairing {
        reset_pairing()?;
        println!("Pairing token reset. Restart without --reset-pairing to generate a new token.");
        return Ok(());
    }

    let config = if let Some(token) = args.pair_token {
        let config = Config { pair_token: token };
        save_config(&config)?;
        config
    } else {
        match load_config()? {
            Some(existing) => existing,
            None => {
                let token = Uuid::new_v4().to_string();
                let config = Config { pair_token: token };
                save_config(&config)?;
                config
            }
        }
    };

    println!("Pairing token: {}", config.pair_token);
    if let Ok(path) = config_path() {
        println!("Config path: {}", path.display());
    }
    if let Some(input) = args.input_file {
        println!("Input file: {}", input);
    }
    println!("Listening on 127.0.0.1:{}", args.port);
    println!("(HTTP server and audio capture not yet implemented)");

    Ok(())
}
