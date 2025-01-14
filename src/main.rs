use clap::Parser;
use colored::Colorize;
use env_logger::Env;
use log::{debug, error, info, warn};
use rusqlite::{Connection, Result};
use std::cmp::PartialEq;
use std::fmt::format;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;
use std::{env, path::Path};
use text_io::read;

mod libfukushuu;

use crate::libfukushuu::db;
use crate::libfukushuu::shitsumon::{get_question_cards, init_questions, rand_category};

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
                    if (num > choices_count as usize) {
                        println!(
                            "{}",
                            format!("There are only {} options available!", choices_count)
                                .bright_red()
                        );
                        Choice::DontKnow
                    } else {
                        Choice::Option(num)
                    }
                }
                Err(_) => Choice::DontKnow,
            },
        }
    }
    fn from_usize(choices_count: u32, input: usize) -> Result<Choice, ()> {
        if (input >= choices_count as usize) {
            Err(())
        } else {
            Ok(Choice::Option(input + 1))
        }
    }
}

fn main() {
    //INIT START
    let args = Args::parse();
    let question_count = args.question_count;
    let choices_count = args.choices_count;
    env_logger::Builder::from_env(Env::default().default_filter_or(args.log_level)).init();

    let db_path = args.db.unwrap_or(PathBuf::from("flashcards.db"));
    let conn = db::create_or_open(db_path).unwrap();
    debug!("[DB] Database Connection Successful!");

    let category = match rand_category(&conn) {
        Some(category) => category,
        None => {
            warn!("[Setup] No categories found.");
            println!(
                "{}",
                "No categories found. Come back when you have added some cards to the database!"
                    .yellow()
            );
            finish(conn, Ok(()));
            exit(0)
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
        for (i, option) in options.iter().enumerate() {
            println!("{}{}. {}", indent, format!("{}", i + 1).bold(), option)
        }

        let correct_choice = match Choice::from_usize(choices_count, correct) {
            Ok(choice) => choice,
            Err(_) => {
                finish(conn, Err("Cannot convert usize to Choice!"));
                exit(1)
            }
        };

        print!(
            "{} ",
            "Answer (1-4, q to quit prematurely and anything else if you don't know):".cyan()
        );
        let choice_string: String = read!("{}\n");
        let choice = Choice::from_str(choices_count, choice_string.as_str());
        debug!("choice: {:?}", choice);

        match choice {
            Choice::Option(num) => {
                if choice == correct_choice {
                    incr_and_print!(questions[idx - 1]);
                } else {
                    decr_and_print!(questions[idx - 1]);
                    println!(
                        "{}",
                        format!("The correct choice was {:?}.", correct_choice).green()
                    )
                }
            }
            Choice::DontKnow => {
                decr_and_print!(questions[idx - 1]);
                println!(
                    "{}",
                    format!("The correct choice was {:?}.", correct_choice).green()
                )
            }
            Choice::Quit => {
                println!("{}", "Quitting Early!".cyan());
                finish(conn, Ok(()));
                exit(0)
            }
        }
    }

    finish(conn, Ok(()));
}

fn finish(conn: Connection, to_error: Result<(), &str>) {
    db::close_db(conn).unwrap();
    exit(match to_error {
        Ok(_) => 0,
        Err(msg) => {
            error!("Need to exit with cause: {}", msg);
            1
        }
    });
}
