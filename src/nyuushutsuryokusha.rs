use colored::Colorize;
use env_logger::Env;
use log::{error, info};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use rusqlite::Error;
use serde::{Deserialize, Serialize};
mod libfukushuu;
use crate::libfukushuu::db;
use crate::libfukushuu::db::{Card, Category, Pool};

#[derive(Parser, Debug)]
#[command(name = "入出力者 (Nyūshutsuryokusha)")]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "info")]
    log_level: String,
    #[arg(short, long, default_value = "false")]
    refresh_db: bool,
    #[arg(short, long, value_name = "FILE", default_value = "flashcards.db")]
    db: Option<PathBuf>,

    json: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Import,
    Export,
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
    category_name: Option<String>,
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

    let json_file = match args.json {
        Some(f) => f,
        None => {
            error!("{}", "From file not specified!".red());
            std::process::exit(1);
        }
    };
    let db_path = match args.db {
        Some(d) => d,
        None => {
            error!("{}", "Database file not specified!".red());
            std::process::exit(1);
        }
    };
    info!(
        "{}",
        format!("File at {:?} and Database at {:?}", json_file, db_path).cyan()
    );
    let db = match db::create_or_open(db_path) {
        Ok(d) => d,
        Err(e) => {
            error!("{}{}", "Unable to open Database: ".red(), e);
            std::process::exit(1);
        }
    };

    let json = std::fs::read_to_string(json_file).unwrap();
    let content: FukushuuJson = match serde_json::from_str(json.as_str()) {
        Ok(c) => c,
        Err(error) => {
            error!("{}", format!("Malformed JSON: {}!", error).red());
            db::close_db(db).unwrap();
            std::process::exit(1);
        }
    };

    match args.command {
        Commands::Import => {
            info!(
                "{}",
                format!(
                    "Importing data... ({} Categories)",
                    content.categories.len()
                )
                .blue()
            );

            macro_rules! check_exists {
                ($to_match: expr, $err_msg: expr) => {
                    match $to_match {
                        Ok(_) => true,
                        Err(e) => match e {
                            Error::QueryReturnedNoRows => false,
                            _ => {
                                error!("{}", format!($err_msg, e).red());
                                false
                            }
                        },
                    }
                };
            }

            content.categories.iter().for_each(|category| {
                info!(
                    "{}",
                    format!(
                        "├ Category: {} ({} Pools)",
                        category.name,
                        category.pools.len()
                    )
                    .blue()
                );
                let category_exists = check_exists!(
                    Category::get_one(&db, &category.name),
                    "Error accessing Categories: {}!"
                );
                if !category_exists {
                    Category::add(
                        &db,
                        Category {
                            name: category.name.clone(),
                        },
                    )
                        .unwrap();
                }

                category.pools.iter().for_each(|pool| {
                    info!(
                        "{}",
                        format!("│ ├ Pool: {} ({} Cards)", pool.id, pool.cards.len()).blue()
                    );
                    let pool_exists =
                        check_exists!(Pool::get_by_id(&db, pool.id), "Error accessing Pools: {}!");
                    if !pool_exists {
                        Pool::add(
                            &db,
                            Pool {
                                id: pool.id,
                                category_name: Some(category.name.clone()),
                            },
                        )
                            .unwrap();
                    }
                    pool.cards.iter().for_each(|card| {
                        if validate_card(card) {
                            Card::add(
                                &db,
                                Card {
                                    id: None,
                                    front: card.front.clone().unwrap_or_default(),
                                    back: card.back.clone().unwrap_or_default(),
                                    front_image: card.front_image.clone().unwrap_or_default(),
                                    back_image: card.back_image.clone().unwrap_or_default(),
                                    score: None,
                                    pool_id: Some(pool.id),
                                    category_name: Some(category.name.clone()),
                                },
                            )
                                .unwrap();
                            info!("{} {}", "│ │".blue(), format!("├ Card: {:?}", card).green());
                        } else {
                            error!(
                                "{} {}",
                                "│ │".blue(),
                                format!(
                            "├ ✘ Card: {:?} (Missing `front`&`front_image` or `back`&`back_image`)",
                            card
                        )
                                .red()
                                .strikethrough()
                            );
                        }
                    });
                });
            });
        }
        Commands::Export => {
            todo!()
        }
    }

    db::close_db(db).unwrap()
}

fn validate_card(card: &CardJson) -> bool {
    if (!card.front.is_none() | !card.front_image.is_none())
        && (!card.back.is_none() | !card.back_image.is_none())
    {
        true
    } else {
        false
    }
}
