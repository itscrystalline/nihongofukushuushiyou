use crate::libfukushuu::db::{Card, Category, Pool};
use log::{debug, warn};
use rand::rng;
use rand::seq::{IndexedRandom, SliceRandom};
use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::process::exit;
use std::time::Instant;

macro_rules! fetch_pool_cards_and_cache {
    ($conn:expr, $pool_id:expr, $cached_pool_id:expr) => {
        match Card::get_in_pool($conn, $pool_id) {
            Ok(cards) => {
                $cached_pool_id = Some($pool_id);
                Some(cards)
            }
            Err(_) => {
                warn!("[Setup] Cannot fetch cards in pool {}", $pool_id);
                None
            }
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

macro_rules! card_face_tuple {
    ($val1:expr, $val2:expr) => {
        (
            if $val1.is_empty() {
                None
            } else {
                Some($val1)
            },
            if $val2.as_os_str().is_empty() {
                None
            } else {
                Some($val2)
            },
        )
    };
}

#[derive(Debug, Clone)]
pub(crate) struct Question {
    pub card_id: i32,
    pub score: i32,
    pub front: (Option<String>, Option<PathBuf>),
    pub correct_option: (Option<String>, Option<PathBuf>),
    pub incorrect_options: Vec<(Option<String>, Option<PathBuf>)>
}

impl Question {
    fn get_str_from_tuple(tuple: &(Option<String>, Option<PathBuf>)) -> String {
        let mut ret = String::from("(");
        let str_exists = if tuple.0 == None { false } else { true };
        let path_exists = if tuple.1 == None { false } else { true };

        ret.push_str(match &tuple.0 {
            None => "",
            Some(str) => {
                str.as_str()
            }
        });
        if (str_exists && path_exists) {
            ret.push_str(", ")
        }
        ret.push_str(match &tuple.1 {
            None => "",
            Some(path) => {
                path.to_str().unwrap()
            }
        });

        ret.push_str(")");
        ret
    }
    pub fn get_front_str(&self) -> String {
        Self::get_str_from_tuple(&self.front)
    }
    pub fn get_correct_str(&self) -> String {
        Self::get_str_from_tuple(&self.correct_option)
    }
    pub fn get_incorrect_str(&self) -> Vec<String> {
        let map = &self.incorrect_options.iter().map(|opt|
            Self::get_str_from_tuple(opt)
        );
        map.clone().collect()
    }
    fn get_all_options_tuple(&self) -> Vec<(Option<String>, Option<PathBuf>)> {
        let mut vec: Vec<(Option<String>, Option<PathBuf>)> = vec![self.correct_option.clone()];
        vec.extend(self.incorrect_options.clone());
        vec
    }
    pub fn get_all_options(&self) -> Vec<String> {
        let mut vec = vec![self.get_correct_str()];
        vec.extend(self.get_incorrect_str());
        vec
    }
    pub fn get_options_randomize(&self) -> (Vec<String>, usize) {
        let mut opts = self.get_all_options();
        let correct = &self.get_correct_str();
        opts.shuffle(&mut rng());
        let index = opts.iter().position(|r| r == correct).unwrap();
        (opts, index)
    }
    fn set_score(&mut self, conn: &Connection, score: i32) -> Result<i32> {
        match Card::change_score(&conn, self.card_id, score) {
            Ok(score) => {
                self.score = score;
                Ok(score)
            },
            Err(err) => Err(err)
        }
    }
    pub fn get_score(&self, conn: &Connection) -> Result<i32> {
        Ok(Card::get_score(&conn, self.card_id)?.unwrap_or(0))
    }
    pub fn increment_score(&mut self, conn: &Connection) -> Result<i32> {
        self.set_score(conn, self.get_score(&conn)? + 1)
    }
    pub fn decrement_score(&mut self, conn: &Connection) -> Result<i32> {
        self.set_score(conn, self.get_score(&conn)? - 1)
    }
}

pub(crate) fn get_question_cards(conn: &Connection, question_count: i32, category: Category) -> Vec<Card> {
    debug!("[Setup] Obtaining {} questions.", question_count);
    let questions_usize = question_count as usize;
    let mut cards = Vec::with_capacity(questions_usize);
    while cards.len() < questions_usize {
        let pool = rand_pool(&conn, &category).unwrap_or_else(|| {
            warn!("[Setup] No pools found.");
            exit(0)
        });
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

pub(crate) fn init_questions(conn: &Connection, cards: Vec<Card>) -> Result<Vec<Question>> {
    let now = Instant::now();
    let mut questions: Vec<Question> = Vec::with_capacity(cards.len());
    let mut cached_pool_id: Option<i32> = None;
    let mut cached_pool_cards: Option<Vec<Card>> = None;
    for card in cards {
        let card_id =
            extract_or_continue!(card.id, "[Setup] Card does not have an `id`! Skipping...");
        let pool_id = extract_or_continue!(
            card.pool_id,
            "[Setup] Card does not have a `pool_id`! Skipping..."
        );

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

        let mut backside: Vec<(Option<String>, Option<PathBuf>)> = cards
            .iter()
            .map(|c| {
                let back_text = c.back.clone();
                let back_img = c.back_image.clone();
                card_face_tuple!(back_text, back_img)
            })
            .collect();
        backside.shuffle(&mut rng());

        questions.push(Question {
            card_id,
            score: card.score.unwrap_or(0),
            front: card_face_tuple!(card.front, card.front_image),
            correct_option: card_face_tuple!(card.back, card.back_image),
            incorrect_options: backside[..3].to_vec(),
        })
    }

    debug!(
        "[Setup] Initialized questions in {} ms.",
        now.elapsed().as_millis()
    );
    Ok(questions)
}

pub(crate) fn rand_category(conn: &Connection) -> Option<Category> {
    let categories = Category::get_all(conn).unwrap();
    match categories.choose(&mut rng()) {
        None => None,
        Some(category) => Some(category.clone()),
    }
}

pub(crate) fn rand_pool(conn: &Connection, category: &Category) -> Option<Pool> {
    let pools = Pool::get_all_in_category(&conn, category.name.clone()).unwrap();
    match pools.choose(&mut rng()) {
        None => None,
        Some(pool) => Some(pool.clone()),
    }
}
