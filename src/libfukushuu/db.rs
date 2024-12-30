use log::{debug, error, info, warn};
use rusqlite::{params, Connection, DatabaseName, Result, Row};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Category {
    pub name: String,
}
#[derive(Debug, Clone)]
pub struct Pool {
    pub id: i32,
    pub category_name: Option<String>,
}
#[derive(Debug, Clone)]
pub struct Card {
    pub id: Option<i32>,
    pub front: String,
    pub back: String,
    pub front_image: PathBuf,
    pub back_image: PathBuf,
    pub score: Option<i32>,
    pub pool_id: Option<i32>,
    pub category_name: Option<String>,
}

impl Category {
    pub fn new(connection: &Connection, name: String) -> Result<()> {
        match connection.execute("INSERT INTO Category(name) VALUES (?1)", params![name]) {
            Ok(_) => {
                debug!("[DB] Created new Category '{}'", name);
                Ok(())
            }
            Err(err) => {
                error!("[DB] Error while creating new Category: {:?}", err);
                Err(err)
            }
        }
    }

    pub fn add(connection: &Connection, src: Category) -> Result<()> {
        Self::new(connection, src.name)
    }
    pub fn delete(connection: &Connection, name: String) -> Result<()> {
        match connection.execute("DELETE FROM Category WHERE name = ?1", params![name]) {
            Ok(_) => {
                debug!("[DB] Deleted Category '{}'", name);
                Ok(())
            }
            Err(err) => {
                error!("[DB] Error while deleting Category: {:?}", err);
                Err(err)
            }
        }
    }

    pub fn get_all(connection: &Connection) -> Result<Vec<Category>> {
        let mut statement = connection.prepare("SELECT * FROM Category")?;
        let rows = statement.query_map([], |row|
            Ok(Category { name: row.get(0)? }))?;

        rows.collect()
    }

    pub fn get_one(connection: &Connection, name: String) -> Result<Category> {
        let mut statement = connection.prepare("SELECT * FROM Category WHERE name = :name LIMIT 1")?;
        let row = statement.query_row(&[(":name", &name)], |row| row.get(0))?;

        Ok(Category { name: row })
    }
}
impl Pool {
    pub fn new(connection: &Connection, id: i32, category_name: Option<String>) -> Result<()> {
        let actual_name = category_name.or(Some(String::new())).unwrap();
        match connection.execute(
            "INSERT INTO Category(id, categoryName) VALUES (?1, ?2)",
            params![id, actual_name],
        ) {
            Ok(_) => {
                debug!("[DB] Created new Pool {} with name '{}'", id, actual_name);
                Ok(())
            }
            Err(err) => {
                error!("[DB] Error while creating new Pool {}: {:?}", id, err);
                Err(err)
            }
        }
    }

    pub fn add(connection: &Connection, src: Pool) -> Result<()> {
        Self::new(connection, src.id, src.category_name)
    }
    pub fn delete(connection: &Connection, id: i32) -> Result<()> {
        match connection.execute("DELETE FROM Pool WHERE id = ?1", params![id]) {
            Ok(_) => {
                debug!("[DB] Deleted Pool '{}'", id);
                Ok(())
            }
            Err(err) => {
                error!("[DB] Error while deleting Pool {}: {:?}", id, err);
                Err(err)
            }
        }
    }

    pub fn get_all(connection: &Connection) -> Result<Vec<Pool>> {
        let mut statement = connection.prepare("SELECT * FROM Pool")?;
        let rows = statement.query_map([], |row|
            Ok(Pool {
                id: row.get(0)?,
                category_name: row.get(1)?,
            }))?;

        rows.collect()
    }
    pub fn get_by_id(connection: &Connection, id: i32) -> Result<Pool> {
        let mut statement = connection.prepare("SELECT * FROM Pool WHERE id = :id LIMIT 1")?;
        let row = statement.query_row(&[(":id", &id)], |row|
            Ok(Pool {
                id: row.get(0)?,
                category_name: row.get(1)?,
            }))?;

        Ok(row)
    }
    pub fn get_all_in_category(connection: &Connection, category_name: String) -> Result<Vec<Pool>> {
        let mut statement = connection.prepare("SELECT * FROM Pool WHERE categoryName = :name")?;
        let rows = statement.query_map(&[(":name", &category_name)], |row|
            Ok(Pool {
                id: row.get(0)?,
                category_name: row.get(1)?,
            }))?;

        rows.collect()
    }
}
impl Card {
    fn insert(connection: &Connection, id: i32, front: String, back: String, front_image: PathBuf,
              back_image: PathBuf, score: i32, pool_id: i32, category_name: String) -> Result<()> {
        let front_image_resolved = front_image.into_os_string().into_string().unwrap_or_default();
        let back_image_resolved = back_image.into_os_string().into_string().unwrap_or_default();
        match connection.execute(
            "INSERT INTO \
            Card(id, front, back, frontImage, backImage, score, poolId, categoryName) \
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, front, back, front_image_resolved, back_image_resolved, score, 
                pool_id, category_name],
        ) {
            Ok(_) => {
                debug!("[DB] Created new Card {} in Pool {} in Category {}", id, pool_id, category_name);
                Ok(())
            }
            Err(err) => {
                error!("[DB] Error while creating new Card {}: {:?}", id, err);
                Err(err)
            }
        }
    }

    pub fn add(connection: &Connection, src: Card) -> Result<()> {
        let id = Self::latest_id(connection).unwrap_or(-1) + 1;
        Self::insert(connection, src.id.unwrap_or(id), src.front, src.back, src.front_image, src.back_image,
                     src.score.unwrap_or(0), src.pool_id.unwrap(), src.category_name.unwrap())
    }

    fn latest_id(connection: &Connection) -> Result<i32> {
        let mut statement = connection.prepare("SELECT id FROM Card ORDER BY id DESC LIMIT 1")?;
        statement.query_row([], |row| row.get(0))
    }

    pub fn delete(connection: &Connection, id: i32) -> Result<()> {
        match connection.execute("DELETE FROM Card WHERE id = ?1", params![id]) {
            Ok(_) => {
                debug!("[DB] Deleted Card '{}'", id);
                Ok(())
            }
            Err(err) => {
                error!("[DB] Error while deleting Card {}: {:?}", id, err);
                Err(err)
            }
        }
    }

    fn from_row(row: &Row) -> Result<Card> {
        Ok(Card {
            id: row.get(0)?,
            front: row.get(1)?,
            back: row.get(2)?,
            front_image: PathBuf::from(row.get::<usize, String>(3)?),
            back_image: PathBuf::from(row.get::<usize, String>(4)?),
            score: row.get(5)?,
            pool_id: row.get(6)?,
            category_name: row.get(7)?,
        })
    }

    pub fn get_all(connection: &Connection) -> Result<Vec<Card>> {
        let mut statement = connection.prepare("SELECT * FROM Card")?;
        let rows = statement.query_map([], |row| Self::from_row(row))?;

        rows.collect()
    }

    pub fn get_by_id(connection: &Connection, id: i32) -> Result<Card> {
        let mut statement = connection.prepare("SELECT * FROM Card WHERE id = :id LIMIT 1")?;
        let row = statement.query_row(&[(":id", &id)], |row| Self::from_row(row))?;

        Ok(row)
    }

    pub fn get_in_pool(connection: &Connection, pool_id: i32) -> Result<Vec<Card>> {
        let mut statement = connection.prepare("SELECT * FROM Card WHERE poolId = :poolId")?;
        let rows = statement.query_map(&[(":poolId", &pool_id)], |row| Self::from_row(row))?;

        rows.collect()
    }

    pub fn get_in_category(connection: &Connection, category_name: String) -> Result<Vec<Card>> {
        let mut statement = connection.prepare("SELECT * FROM Card WHERE categoryName = :categoryName")?;
        let rows = statement.query_map(&[(":categoryName", &category_name)], |row| Self::from_row(row))?;

        rows.collect()
    }

    pub fn change_score(connection: &Connection, id: i32, score: i32) -> Result<i32> {
        match connection.execute("UPDATE Card SET score = ?2 WHERE id = ?1", params![id, score]) {
            Ok(_) => Ok(score),
            Err(err) => {
                error!("[DB] Failed to update score for ID {}.", id);
                Err(err)
            }
        }
    }
    pub fn get_score(connection: &Connection, id: i32) -> Result<Option<i32>> {
        let card = Card::get_by_id(&connection, id)?;
        Ok(card.score)
    }
}
pub(crate) fn create_or_open(src: &Path) -> Result<Connection> {
    if src.exists() {
        info!("[DB] Opening existing Database");
        open_db(src)
    } else {
        info!("[DB] Creating new Database");
        create_db(src)
    }
}

pub(crate) fn create_db(dest: &Path) -> Result<Connection> {
    let now = Instant::now();
    let mut db = Connection::open_in_memory()?;
    db = init_db(db)?;
    match db.backup(DatabaseName::Main, dest, None) {
        Ok(_) => {
            debug!(
                "[DB] Creating and Saving took {} ms.",
                now.elapsed().as_millis()
            );
            Ok(db)
        }
        Err(err) => {
            warn!("Failed to create database file: {}", err);
            close_db(db)?;
            Err(err)
        }
    }
}

pub(crate) fn open_db(src: &Path) -> Result<Connection> {
    let now = Instant::now();
    let db = Connection::open(src)?;
    debug!("[DB] Opening took {} ms.", now.elapsed().as_millis());
    Ok(db)
}

pub(crate) fn close_db(connection: Connection) -> Result<()> {
    info!("[DB] Closing Database");
    match connection.close() {
        Ok(_) => Ok(()),
        Err((conn, err)) => {
            error!("[DB] Cannot close connection. Retrying 1/2...");
            match conn.close() {
                Ok(_) => Ok(()),
                Err((conn2, err)) => {
                    error!("[DB] Cannot close connection. Retrying 2/2...");
                    match conn2.close() {
                        Ok(_) => Ok(()),
                        Err(_) => panic!("[DB] Cannot close connection! Aborting.")
                    }
                }
            }
        }
    }
}

fn init_db(conn: Connection) -> Result<Connection> {
    info!("[DB INIT] Creating tables");
    conn.execute(
        "CREATE TABLE Category (
              name TEXT NOT NULL,
              PRIMARY KEY (name)
            )",
        (),
    )?;
    info!("[DB INIT] Created table Category");
    conn.execute(
        "CREATE TABLE Pool (
              id INTEGER NOT NULL PRIMARY KEY,
              categoryName TEXT,
              FOREIGN KEY (categoryName) REFERENCES Category(name) ON DELETE SET NULL ON UPDATE CASCADE
            )", (),
    )?;
    info!("[DB INIT] Created table Pool");
    conn.execute(
        "CREATE TABLE Card (
              id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
              front TEXT NOT NULL,
              back TEXT NOT NULL,
              frontImage TEXT NOT NULL,
              backImage TEXT NOT NULL,
              score INTEGER NOT NULL,
              poolId INTEGER,
              categoryName TEXT,
              FOREIGN KEY (poolId) REFERENCES Pool(id) ON DELETE SET NULL ON UPDATE CASCADE,
              FOREIGN KEY (categoryName) REFERENCES Category(name) ON DELETE SET NULL ON UPDATE CASCADE
            )", (),
    )?;
    info!("[DB INIT] Created table Card");
    conn.execute("CREATE INDEX Card_poolId_idx ON Card(poolId)", ())?;
    info!("[DB INIT] Created index Card_poolId_idx");
    conn.execute(
        "CREATE INDEX Card_categoryName_idx ON Card(categoryName)",
        (),
    )?;
    info!("[DB INIT] Created index Card_categoryName_idx");
    conn.execute(
        "CREATE INDEX Pool_categoryName_idx ON Pool(categoryName)",
        (),
    )?;
    info!("[DB INIT] Created index Pool_categoryName_idx");
    info!("[DB INIT] Database Creation Successful!");
    
    Ok(conn)
}
