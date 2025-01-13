use colored::Colorize;
use env_logger::Env;
use log::{error, info};
use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "入力者 (Nyuuryokusha)")]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "error")]
    log_level: String,
    #[arg(short, long, default_value = "false")]
    refresh_db: bool,
    #[arg(short, long, value_name = "FILE", default_value = "flashcards.db")]
    db: Option<PathBuf>,
    from: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FukushuuJson {
    categories: Vec<CategoryJson>,
}
#[derive(Serialize, Deserialize, Debug)]
struct CategoryJson {
    name: String,
    pools: Vec<PoolJson>,
}
#[derive(Serialize, Deserialize, Debug)]
struct PoolJson {
    id: i32,
    name: Option<String>,
    cards: Vec<CardJson>,
}
#[derive(Serialize, Deserialize, Debug)]
struct CardJson {
    front: Option<String>,
    back: Option<String>,
    front_image: Option<PathBuf>,
    back_image: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or(args.log_level)).init();

    let from_file = match args.from {
        Some(f) => f,
        None => {
            error!("{}", "From file not specified!".red());
            std::process::exit(1);
        }
    };
    info!(
        "{}",
        format!("Reading from file {:?} to db {:?}", from_file, args.db).cyan()
    );

    let json = std::fs::read_to_string(from_file).unwrap();
    let content: FukushuuJson = match serde_json::from_str(json.as_str()) {
        Ok(c) => c,
        Err(error) => {
            error!("{}", format!("Malformed JSON: {}!", error).red());
            std::process::exit(1);
        }
    };

    println!("{:?}", content);
}
