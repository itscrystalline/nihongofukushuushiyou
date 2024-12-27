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
    let db_path = Path::new("flashcards.db");
    let conn = db::create_or_open(db_path).unwrap();
    debug!("[DB] Database Connection Successful!");

    let category = rand_category(&conn);
    debug!("[Setup] Picked category {:?}", category);

    let pool = rand_pool(&conn, category);
    debug!("[Setup] Picked pool {:?}", pool);

    let cards = Card::get_in_pool(&conn, pool.id).unwrap();
    debug!("[Setup] This pool has {} Cards.", cards.len());

    let questions = init_questions(cards);

    db::close_db(conn).unwrap()
}

fn init_questions(cards: Vec<Card>) -> Vec<Question> {
    todo!()
}

fn rand_category(conn: &Connection) -> Category {
    let categories = Category::get_all(conn).unwrap();
    let pick = categories.choose(&mut rng()).unwrap();
    pick.clone()
}

fn rand_pool(conn: &Connection, category: Category) -> Pool {
    let pools = Pool::get_all_in_category(&conn, category.name).unwrap();
    let pick = pools.choose(&mut rng()).unwrap();
    pick.clone()
}
