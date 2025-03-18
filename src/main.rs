use clap::Parser;
use colored::Colorize;
use env_logger::Env;
use libfukushuu::shitsumon::{category, OptionPair, Question};
use log::{debug, warn};
use rusqlite::{Connection, Result};
use std::cmp::PartialEq;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "gui")]
mod gui;

mod libfukushuu;

use crate::libfukushuu::db;
use crate::libfukushuu::shitsumon::{get_question_cards, init_questions};

#[derive(Debug, PartialEq)]
enum Choice {
    Option(usize),
    DontKnow,
    Quit,
}

#[derive(Parser, Debug)]
#[command(name = "日本語復習しよう！ (Nihongofukushūshiyō!)")]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE", default_value = "flashcards.db")]
    db: Option<PathBuf>,
    #[arg(long)]
    category: Option<String>,
    #[arg(short, long, default_value = "20")]
    question_count: u32,
    #[arg(short, long, default_value = "4")]
    choices_count: u32,
    #[arg(short, long, default_value = "error")]
    log_level: String,
}

impl Choice {
    fn from_str(choices_count: u32, input: &str) -> Choice {
        match input {
            "q" => Choice::Quit,
            input => match input.parse::<usize>() {
                Ok(num) => {
                    if num > choices_count as usize {
                        println!(
                            "{}",
                            format!("There are only {} options available!", choices_count)
                                .bright_red()
                        );
                        Choice::DontKnow
                    } else {
                        Choice::Option(num - 1)
                    }
                }
                Err(_) => Choice::DontKnow,
            },
        }
    }
}

#[derive(Debug, Error)]
enum Error {
    #[error("no categories!")]
    NoCategories,
    #[cfg(feature = "kittygfx")]
    #[error("Cannot read image")]
    ImageRead(#[from] io::Error),
    #[cfg(feature = "kittygfx")]
    #[error("cannot decode image")]
    ImageDecode(#[from] image::ImageError),
    #[cfg(feature = "gui")]
    #[error("cannot initialize gui")]
    GuiInitialization(#[from] eframe::Error),
}

fn main() -> Result<(), Error> {
    //INIT START
    let args = Args::parse();
    let question_count = args.question_count;
    let choices_count = args.choices_count;
    env_logger::Builder::from_env(Env::default().default_filter_or(args.log_level)).init();

    let db_path = args.db.unwrap_or(PathBuf::from("flashcards.db"));
    let conn = db::create_or_open(db_path).unwrap();
    debug!("[DB] Database Connection Successful!");

    let category = match category(&conn, args.category.as_deref()) {
        Some(category) => category,
        None => {
            warn!("[Setup] No categories found.");
            println!(
                "{}",
                "No categories found. Come back when you have added some cards to the database!"
                    .yellow()
            );
            return finish(conn, Err(Error::NoCategories));
        }
    };
    debug!("[Setup] Picked category {:?}", category);
    println!(
        "{}",
        format!(
            "==========> {} ({} questions) <==========",
            category.name, question_count
        )
        .cyan()
    );

    let cards = get_question_cards(&conn, question_count, category);
    debug!("[Setup] Cards: {:?}", cards);

    let mut questions = init_questions(&conn, cards, choices_count).unwrap();
    debug!("[Setup] Questions: {:?}", questions.len());

    // INIT DONE
    #[cfg(feature = "cli")]
    cli::cli_loop(&conn, &mut questions, question_count, choices_count)?;
    #[cfg(feature = "gui")]
    gui::init_gui(&conn, &mut questions, question_count, choices_count)?;

    finish(conn, Ok(()))
}

fn finish(conn: Connection, to_error: Result<(), Error>) -> Result<(), Error> {
    db::close_db(conn).unwrap();
    to_error
}
