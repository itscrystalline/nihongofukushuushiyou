use crate::db::{Card, Category, Pool};
use crate::question::Question;
use log::debug;
use rand::{rng, seq::IndexedRandom};
use rusqlite::{Connection, Result};
use std::{env, ops::Deref, path::Path};

mod db;
mod question;

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let question_count = 20;
    
    let db_path = Path::new("flashcards.db");
    let conn = db::create_or_open(db_path).unwrap();
    debug!("[DB] Database Connection Successful!");

    let category = rand_category(&conn);
    debug!("[Setup] Picked category {:?}", category);

    let pool = rand_pool(&conn, &category);
    debug!("[Setup] Picked pool {:?}", pool);

    let cards = get_question_cards(&conn, question_count, &category, pool)

    let questions = init_questions(cards);

    db::close_db(conn).unwrap()
}

fn get_question_cards(conn: &Connection, question_count: i32, category: &Category, pool: Pool) -> Vec<Question> {
    let cards = Vec::with_capacity(question_count);
    debug!("[Setup] This pool has {} Cards.", cards.len());
    if cards.len() < question_count {
      debug!("[Setup] ...which is less than the required {} cards. finding more...", question_count);
      let new_pool = rand_pool(&conn, &category);
      let additional_cards = Card::get_in_pool(&conn, new_pool.id).unwrap();m
    }
    
    cards
}

fn init_questions(cards: Vec<Card>) -> Vec<Question> {
    todo!()
}

fn rand_category(conn: &Connection) -> Category {
    let categories = Category::get_all(conn).unwrap();
    let pick = categories.choose(&mut rng()).unwrap();
    pick.clone()
}

fn rand_pool(conn: &Connection, category: &Category) -> Pool {
    let pools = Pool::get_all_in_category(&conn, category.name).unwrap();
    let pick = pools.choose(&mut rng()).unwrap();
    pick.clone()
}
