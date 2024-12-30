use console_menu::{color, Menu, MenuOption, MenuProps};
use log::{debug, warn};
use std::process::exit;
use std::{env, path::Path};
use rusqlite::{Connection, Result};

mod libfukushuu;
mod nyuuryokusha;

use crate::libfukushuu::shitsumon::Question;
use libfukushuu::db;
use libfukushuu::shitsumon::{get_question_cards, init_questions, rand_category};

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
            db::close_db(conn).unwrap();
            exit(0)
        }
    };
    debug!("[Setup] Picked category {:?}", category);

    let cards = get_question_cards(&conn, question_count, category);

    let questions = init_questions(&conn, cards).unwrap();
    debug!("[Setup] Questions: {:?}", questions);

    // INIT DONE

    for question in questions {
        let mut menu = build_menu(&conn, &question);
        menu.show();
    }

    db::close_db(conn).unwrap()
}

fn build_menu(conn: &Connection, question: &Question) -> Menu {
    let options = question.get_all_options();
    let menu_options = options.iter().map(|opt| {
        let owned_opt = opt.clone();
        MenuOption::new(owned_opt.clone().as_str(), move || {
            println!("{}", owned_opt.as_str());
            question.increment_score(&conn);
        });
        todo!()
    }).collect();

    Menu::new(menu_options, MenuProps {
        title: question.get_front_str().as_str(),
        message: "",
        fg_color: color::BLACK,
        bg_color: color::BLUE,
        msg_color: Some(color::DARK_GRAY),
        ..MenuProps::default()
    })
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

