use crate::db::{Card, Category, Pool};
use log::debug;
use rand::rng;
use rand::seq::IndexedRandom;
use rusqlite::{Connection, Result};
use std::ops::Deref;
use std::path::Path;

mod db;

fn main() {
    env_logger::init();
    let db_path = Path::new("flashcards.db");
    let conn = db::create_or_open(db_path).unwrap();
    debug!("[DB] Database Connection Successful!");

    let category = rand_category(&conn);
    debug!("[Setup] Picked category {:?}", category);

    let pool = rand_pool(&conn, category);
    debug!("[Setup] Picked pool {:?}", pool);

    let cards = Card::get_in_pool(&conn, pool.id).unwrap();
    debug!("[Setup] Cards: {:?}", cards);

    db::close_db(conn).unwrap()
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
