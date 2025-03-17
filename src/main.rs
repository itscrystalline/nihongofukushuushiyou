use clap::Parser;
use colored::Colorize;
use env_logger::Env;
use kitty_image::{Action, Command, WrappedCommand};
use libfukushuu::shitsumon::{category, OptionPair};
use log::{debug, warn};
use rusqlite::{Connection, Result};
use std::cmp::PartialEq;
use std::io;
use std::path::PathBuf;
use text_io::read;
use thiserror::Error;

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
    #[error("Cannot read image")]
    ImageRead(#[from] io::Error),
    #[error("cannot decode image")]
    ImageDecode(#[from] image::ImageError),
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

    for idx in 1..questions.len() + 1 {
        macro_rules! incr_and_print {
            ($to_incr: expr) => {
                let score = $to_incr.increment_score(&conn).unwrap();
                println!(
                    "{}",
                    format!("Correct!: {} -> {}", score - 1, score).bright_green()
                )
            };
        }
        macro_rules! decr_and_print {
            ($to_incr: expr) => {
                let score = $to_incr.decrement_score(&conn).unwrap();
                println!(
                    "{}",
                    format!("Incorrect!: {} -> {}", score + 1, score).bright_red()
                )
            };
        }

        let leading = format!("{}/{}. ", idx, question_count);
        println!(
            "{}{}",
            leading.cyan(),
            format!(
                "{:?} ({})",
                questions[idx - 1].front,
                questions[idx - 1].score
            )
            .black()
            .bold()
            .on_white()
        );
        let (options, correct) = questions[idx - 1].get_options_randomize();

        let indent = " ".repeat(leading.len());
        for (i, OptionPair(str, img)) in options.iter().enumerate() {
            print!("{}{}. ", indent, format!("{}", i + 1).bold());
            if let Some(string) = str {
                println!("{}", string);
            }
            if let Some(image_path) = img {
                debug!("path at {image_path:?}");
                let (width, height) = image::image_dimensions(image_path)?;
                let action = Action::TransmitAndDisplay(
                    kitty_image::ActionTransmission {
                        format: kitty_image::Format::Png,
                        medium: kitty_image::Medium::File,
                        width,
                        height,
                        ..Default::default()
                    },
                    kitty_image::ActionPut {
                        x_offset: 10 * leading.len() as u32,
                        ..Default::default()
                    },
                );
                let command =
                    WrappedCommand::new(Command::with_payload_from_path(action, image_path));
                println!("{command}");
                print!("{}", "\n".repeat(height as usize / 20));
            }
        }

        print!(
            "{} ",
            "Answer (1-4, q to quit prematurely and anything else if you don't know):".cyan()
        );
        let choice_string: String = read!("{}\n");
        let choice = Choice::from_str(choices_count, choice_string.as_str());
        debug!("choice: {:?}", choice);

        match choice {
            Choice::Option(num) => {
                if num == correct {
                    incr_and_print!(questions[idx - 1]);
                } else {
                    decr_and_print!(questions[idx - 1]);
                    println!(
                        "{}",
                        format!("The correct choice was {:?}.", correct).green()
                    )
                }
            }
            Choice::DontKnow => {
                decr_and_print!(questions[idx - 1]);
                println!(
                    "{}",
                    format!("The correct choice was {:?}.", correct).green()
                )
            }
            Choice::Quit => {
                println!("{}", "Quitting Early!".cyan());
                return finish(conn, Ok(()));
            }
        }
    }

    finish(conn, Ok(()))
}

fn finish(conn: Connection, to_error: Result<(), Error>) -> Result<(), Error> {
    db::close_db(conn).unwrap();
    to_error
}
