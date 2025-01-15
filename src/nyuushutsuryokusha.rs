use clap::{Parser, Subcommand};
use colored::Colorize;
use env_logger::Env;
use log::{error, info};
use rusqlite::Error;
use serde::{Deserialize, Serialize};
use std::fmt::format;
use std::path::PathBuf;
use std::time::Instant;
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
impl CategoryJson {
    fn from(cate: &Category) -> CategoryJson {
        CategoryJson {
            name: cate.name.clone(),
            pools: vec![],
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
struct PoolJson {
    id: i32,
    category_name: Option<String>,
    cards: Vec<CardJson>,
}
impl PoolJson {
    fn from(pool: &Pool) -> PoolJson {
        PoolJson {
            id: pool.id,
            category_name: pool.category_name.clone(),
            cards: vec![],
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
struct CardJson {
    id: Option<i32>,
    front: Option<String>,
    back: Option<String>,
    front_image: Option<PathBuf>,
    back_image: Option<PathBuf>,
    score: Option<i32>,
}
macro_rules! empty_none_or_some {
    ($condition: expr, $some_value: expr) => {
        match $condition {
            true => None,
            false => Some($some_value),
        }
    };
}
impl CardJson {
    fn from(card: &Card) -> CardJson {
        CardJson {
            id: card.id,
            front: empty_none_or_some!(card.front.is_empty(), card.front.clone()),
            back: empty_none_or_some!(card.back.is_empty(), card.back.clone()),
            front_image: empty_none_or_some!(card.front_image.clone().into_os_string().is_empty(), card.front_image.clone()),
            back_image: empty_none_or_some!(card.back_image.clone().into_os_string().is_empty(), card.back_image.clone()),
            score: card.score,
        }
    }
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

    match args.command {
        Commands::Import => {
            let json = std::fs::read_to_string(json_file).unwrap();
            let content: FukushuuJson = match serde_json::from_str(json.as_str()) {
                Ok(c) => c,
                Err(error) => {
                    error!("{}", format!("Malformed JSON: {}!", error).red());
                    db::close_db(db).unwrap();
                    std::process::exit(1);
                }
            };

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
                                    id: card.id,
                                    front: card.front.clone().unwrap_or_default(),
                                    back: card.back.clone().unwrap_or_default(),
                                    front_image: card.front_image.clone().unwrap_or_default(),
                                    back_image: card.back_image.clone().unwrap_or_default(),
                                    score: card.score,
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
            let start = Instant::now();

            let mut exported = FukushuuJson {
                categories: vec![],
            };

            let available_categories = Category::get_all(&db).unwrap();
            available_categories.iter().enumerate().for_each(|(i, category)| {
                info!("{}", format!("Exporting Category {}/{}", i + 1, available_categories.len()).blue());
                let mut category = CategoryJson::from(category);
                let pools_in_category = Pool::get_all_in_category(&db, &category.name).unwrap();
                pools_in_category.iter().enumerate().for_each(|(j, pool)| {
                    info!("  {}", format!("Exporting Pool {}/{}", j + 1, pools_in_category.len()).blue());
                    let mut pool = PoolJson::from(pool);
                    let cards_in_pool = Card::get_in_pool(&db, pool.id).unwrap();
                    cards_in_pool.iter().enumerate().for_each(|(k, card)| {
                        info!("    {}", format!("Exporting Card {}/{}", k + 1, cards_in_pool.len()).green());
                        pool.cards.push(CardJson::from(card));
                    });
                    category.pools.push(pool);
                });
                exported.categories.push(category);
            });

            let json_exported = serde_json::to_string(&exported).unwrap();
            std::fs::write(json_file, json_exported).unwrap();
            info!("{}", format!("Export Complete in {} ms!", start.elapsed().as_millis()).green());
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
