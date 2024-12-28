use crate::db::{Card, Category, Pool};
use crate::question::Question;
use log::debug;
use rand::{rng, seq::IndexedRandom};
use rusqlite::{Connection, Result};
use std::{env, path::Path};

mod db;
mod question;

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let question_count = handle_args(args);

    let db_path = Path::new("flashcards.db");
    let conn = db::create_or_open(db_path).unwrap();
    debug!("[DB] Database Connection Successful!");

    let category = rand_category(&conn);
    debug!("[Setup] Picked category {:?}", category);

    let cards = get_question_cards(&conn, question_count, category);

    let questions = init_questions(cards);

    db::close_db(conn).unwrap()
}

fn handle_args(args: Vec<String>) -> i32 {
    let question_count;
    if args.len() > 1 {
        question_count = match &args[1].parse::<i32>() {
            Ok(count) => *count,
            Err(_) => 20
        };
    } else {
        question_count = 20;
    }

    question_count
}


fn get_question_cards<'a>(conn: &Connection, question_count: i32, category: Category) -> Vec<Card> {
    let questions_usize = question_count as usize;
    let mut cards = Vec::with_capacity(questions_usize);
    while cards.len() < questions_usize {
        let pool = rand_pool(&conn, &category);
        debug!("[Setup] Picked pool {:?}", pool);
        let pool_cards = Card::get_in_pool(&conn, pool.id).unwrap();
        debug!("[Setup] which contains {} cards.", pool_cards.len());

        if cards.len() + pool_cards.len() > questions_usize {
            let to_keep = questions_usize - cards.len();
            debug!("[Setup] Cards is too full for this pool. Keeping {} elements.", to_keep);
            let to_keep_vec = pool_cards[..to_keep].to_vec();
            cards.extend(to_keep_vec)
        } else {
            cards.extend(pool_cards);
        }
    }

    cards
}

fn init_questions(cards: Vec<Card>) -> Vec<Question> {
    let mut cached_pool_cards: Vec<Card>;
    for card in cards {}
    todo!()
}

fn rand_category(conn: &Connection) -> Category {
    let categories = Category::get_all(conn).unwrap();
    let pick = categories.choose(&mut rng()).unwrap();
    pick.clone()
}

fn rand_pool(conn: &Connection, category: &Category) -> Pool {
    let pools = Pool::get_all_in_category(&conn, category.name.clone()).unwrap();
    let pick = pools.choose(&mut rng()).unwrap();
    pick.clone()
}
