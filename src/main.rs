use colored::Colorize;
use log::{debug, error, info, warn};
use rusqlite::{Connection, Result};
use std::process::exit;
use std::str::FromStr;
use std::{env, path::Path};
use text_io::read;

mod libfukushuu;
mod nyuuryokusha;

use crate::libfukushuu::db;
use crate::libfukushuu::shitsumon::{get_question_cards, init_questions, rand_category};

#[derive(Debug)]
enum Choice {
    One,
    Two,
    Three,
    Four,
    DontKnow,
    Quit,
}

impl FromStr for Choice {
    type Err = ();
    fn from_str(input: &str) -> Result<Choice, ()> {
        match input {
            "1" => Ok(Choice::One),
            "2" => Ok(Choice::Two),
            "3" => Ok(Choice::Three),
            "4" => Ok(Choice::Four),
            "q" => Ok(Choice::Quit),
            _ => Ok(Choice::DontKnow),
        }
    }
}

impl Choice {
    fn from_usize(input: usize) -> Result<Choice, ()> {
        match input {
            0 => Ok(Choice::One),
            1 => Ok(Choice::Two),
            2 => Ok(Choice::Three),
            3 => Ok(Choice::Four),
            _ => Err(()),
        }
    }
}

fn main() {
    //INIT START

    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let question_count = handle_args(args);

    let db_path = Path::new("flashcards.db");
    let conn = db::create_or_open(db_path).unwrap();
    debug!("[DB] Database Connection Successful!");

    let category = match rand_category(&conn) {
        Some(category) => category,
        None => {
            warn!("[Setup] No categories found. Come back when you have added some cards to the database!");
            finish(conn, Ok(()));
            exit(0)
        }
    };
    debug!("[Setup] Picked category {:?}", category);
    println!("{}", format!("==========> {} ({} questions) <==========", category.name, question_count).cyan());

    let cards = get_question_cards(&conn, question_count, category);

    let questions = init_questions(&conn, cards).unwrap();
    debug!("[Setup] Questions: {:?}", questions);

    // INIT DONE

    for (idx, question) in questions.iter().enumerate() {
        let leading = format!("{}/{}. ", idx + 1, question_count);
        println!("{}{}",
                 leading.cyan(),
                 format!("{:?} ({})", question.front, question.score).black().bold().on_white()
        );
        let (options, correct) = question.get_options_randomize();

        let indent = " ".repeat(leading.len());
        for (i, option) in options.iter().enumerate() {
            println!("{}{}. {}", indent,
                     format!("{}", i + 1).bold(),
                     option
            )
        }

        let _correct_choice = match Choice::from_usize(correct) {
            Ok(choice) => choice,
            Err(_) => {
                finish(conn, Err("Cannot convert usize to Choice!"));
                exit(1)
            }
        };

        print!("{} ", "Answer (1-4, q to quit prematurely and anything else if you don't know):".cyan());
        let choice_string: String = read!("{}\n");
        let choice = Choice::from_str(choice_string.as_str()).unwrap();

        info!("choice: {:?}", choice)
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

fn handle_args(args: Vec<String>) -> i32 {
    let question_count;
    if args.len() > 1 {
        question_count = match &args[1].parse::<i32>() {
            Ok(count) => *count,
            Err(_) => 20,
        };
    } else {
        question_count = 20;
    }

    question_count
}

