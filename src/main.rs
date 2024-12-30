use log::{debug, warn};
use std::process::exit;
use std::{env, path::Path};

mod libfukushuu;
mod importer;

use libfukushuu::db;
use libfukushuu::question::{get_question_cards, init_questions, rand_category};

fn main() {
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
            db::close_db(conn).unwrap();
            exit(0)
        }
    };
    debug!("[Setup] Picked category {:?}", category);

    let cards = get_question_cards(&conn, question_count, category);

    let questions = init_questions(&conn, cards).unwrap();
    debug!("[Setup] Questions: {:?}", questions);

    db::close_db(conn).unwrap()
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

