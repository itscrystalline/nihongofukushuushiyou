use crate::db::{Card, Category, Pool};
use crate::question::Question;
use log::{debug, warn};
use rand::seq::SliceRandom;
use rand::{rng, seq::IndexedRandom};
use rusqlite::{Connection, Result};
use std::process::exit;
use std::time::Instant;
use std::{env, path::Path};

mod db;
mod question;

macro_rules! fetch_pool_cards_and_cache {
    ($conn:expr, $pool_id:expr, $cached_pool_id:expr) => {
        match Card::get_in_pool($conn, $pool_id) {
            Ok(cards) => {
                $cached_pool_id = Some($pool_id);
                Some(cards)
            },
            Err(_) => {
                warn!("[Setup] Cannot fetch cards in pool {}", $pool_id);
                None
            },
        }
    };
}
macro_rules! extract_or_continue {
    ($field:expr, $warn_msg:expr $(, $args:expr)*) => {
        match $field {
            None => {
                warn!($warn_msg $(, $args)*);
                continue;
            }
            Some(value) => value,
        }
    };
}

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

fn get_question_cards(conn: &Connection, question_count: i32, category: Category) -> Vec<Card> {
    debug!("[Setup] Obtaining {} questions.", question_count);
    let questions_usize = question_count as usize;
    let mut cards = Vec::with_capacity(questions_usize);
    while cards.len() < questions_usize {
        let pool = match rand_pool(&conn, &category) {
            Some(pool) => pool,
            None => {
                warn!("[Setup] No pools found.");
                exit(0)
            }
        };
        debug!("[Setup] Picked pool {:?}", pool);
        let mut pool_cards = Card::get_in_pool(&conn, pool.id).unwrap();
        pool_cards.shuffle(&mut rng());
        debug!("[Setup] ...which contains {} cards.", pool_cards.len());

        if cards.len() + pool_cards.len() > questions_usize {
            let to_keep = questions_usize - cards.len();
            debug!(
                "[Setup] Cards is too full for this pool. Keeping {} elements.",
                to_keep
            );
            let to_keep_vec = pool_cards[..to_keep].to_vec();
            cards.extend(to_keep_vec)
        } else {
            cards.extend(pool_cards);
        }
    }

    cards
}

fn init_questions(conn: &Connection, cards: Vec<Card>) -> Result<Vec<Question>> {
    let now = Instant::now();
    let mut questions: Vec<Question> = Vec::with_capacity(cards.len());
    let mut cached_pool_id: Option<i32> = None;
    let mut cached_pool_cards: Option<Vec<Card>> = None;
    for card in cards {
        let card_id = extract_or_continue!(card.id, "[Setup] Card does not have an `id`! Skipping...");
        let pool_id = extract_or_continue!(card.pool_id, "[Setup] Card does not have a `pool_id`! Skipping...");

        match cached_pool_id {
            None => cached_pool_cards = fetch_pool_cards_and_cache!(&conn, pool_id, cached_pool_id),
            Some(cached) => {
                if cached != pool_id {
                    cached_pool_cards = fetch_pool_cards_and_cache!(&conn, pool_id, cached_pool_id);
                }
            }
        }
        // cached_pool_cards should be Some() by now

        let mut cards = cached_pool_cards.clone().unwrap();
        cards.retain(|c| c.id.unwrap() != card_id);

        let mut backside: Vec<String> = cards.iter().map(
            |c| if c.back.is_empty() {
                let back_image_path = c.back_image.clone().into_os_string().into_string().unwrap();
                back_image_path
            } else {
                c.back.clone()
            }
        ).collect();
        backside.shuffle(&mut rng());

        questions.push(Question {
            card_id,
            front: card.front,
            front_image: card.front_image,
            correct_option: card.back,
            incorrect_options: backside[..3].to_vec(),
        })
    }

    debug!("[Setup] Initialized questions in {} ms.", now.elapsed().as_millis());
    Ok(questions)
}

fn rand_category(conn: &Connection) -> Option<Category> {
    let categories = Category::get_all(conn).unwrap();
    match categories.choose(&mut rng()) {
        None => None,
        Some(category) => Some(category.clone()),
    }
}

fn rand_pool(conn: &Connection, category: &Category) -> Option<Pool> {
    let pools = Pool::get_all_in_category(&conn, category.name.clone()).unwrap();
    match pools.choose(&mut rng()) {
        None => None,
        Some(pool) => Some(pool.clone()),
    }
}
