use core::fmt;

use rusqlite::Result;
use typed_builder::TypedBuilder as Builder;

use crate::{bot::fsm, config::Config, db::utils::query_wrapper};

use super::utils::{self, Header};

pub const TABLE_NAME: &str = "user";

#[derive(Debug, Builder)]
pub struct User {
    pub id: Header<u64>,
    pub name: Header<String>,
    pub manager: Header<String>,
    pub chat_id: Header<i64>,
}

impl User {
    pub fn new() -> Self {
        Self::builder()
            .id(Header::new(0, "id"))
            .name(Header::new("John Doe".into(), "name"))
            .manager(Header::new("Richard Roe".into(), "manager"))
            .chat_id(Header::new(0, "chat_id"))
            .build()
    }

    pub fn create_table(&self) -> Result<()> {
        let conn = utils::open(Config::database_location())?;
        let query = query_wrapper(format!(
            "CREATE TABLE IF NOT EXISTS {} (
            {}  INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT ,
            {}  TEXT NOT NULL UNIQUE,
            {}  TEXT NOT NULL,
            {}  INTEGER NOT NULL UNIQUE
            )",
            TABLE_NAME, self.id.name, self.name.name, self.manager.name, self.chat_id.name,
        ));

        conn.execute(&query, ())?;

        Ok(())
    }

    pub fn insert(&self) -> Result<Self> {
        let conn = utils::open(Config::database_location())?;
        let query = query_wrapper(format!(
            "INSERT INTO {} ({},{},{}) 
            VALUES (?, ?, ?)",
            TABLE_NAME, self.name.name, self.manager.name, self.chat_id.name,
        ));
        conn.execute(
            &query,
            (&self.name.value, &self.manager.value, &self.chat_id.value),
        )?;

        let query = query_wrapper(format!(
            "SELECT * FROM {} ORDER BY {} DESC LIMIT 1",
            TABLE_NAME, self.id.name
        ));

        let mut stmt = conn.prepare(&query)?;
        let mut user_iter = stmt.query_map([], |row| Self::from_row(row))?;

        // Ok(User::new())
        user_iter.next().unwrap()
    }

    pub fn insert_or_update(&self) -> Result<Self> {
        let conn = utils::open(Config::database_location())?;
        let query = query_wrapper(format!(
            "INSERT OR IGNORE INTO {} ({},{},{}) 
                VALUES ('{}', '{}', '{}')",
            TABLE_NAME,
            self.name.name,
            self.manager.name,
            self.chat_id.name,
            self.name.value,
            self.manager.value,
            self.chat_id.value,
        ));
        conn.execute(
            &query,
            (), // (&self.name.value, &self.manager.value, &self.chat_id.value),
        )?;

        let query = query_wrapper(format!(
            "UPDATE {} SET {}='{}', {}='{}' WHERE {}='{}';",
            TABLE_NAME,
            self.name.name,
            self.name.value,
            self.manager.name,
            self.manager.value,
            self.chat_id.name,
            self.chat_id.value
        ));
        conn.execute(
            &query,
            (), // (&self.name.value, &self.manager.value, &self.chat_id.value),
        )?;

        let query = query_wrapper(format!(
            "SELECT * FROM {} ORDER BY {} DESC LIMIT 1",
            TABLE_NAME, self.id.name
        ));

        let mut stmt = conn.prepare(&query)?;
        let mut user_iter = stmt.query_map([], |row| Self::from_row(row))?;

        // Ok(User::new())
        let result = user_iter.next().unwrap();
        log::debug!("user: {:?}", result.as_ref().unwrap());
        result
    }

    pub fn update_one<T, U>(&self, h_update: &Header<T>, h_where: &Header<U>) -> Result<()>
    where
        T: fmt::Display,
        U: fmt::Display,
    {
        let conn = utils::open(Config::database_location())?;
        let query = query_wrapper(format!(
            "UPDATE {} SET {}='{}' WHERE {}='{}'",
            TABLE_NAME, h_update.name, h_update.value, h_where.name, h_where.value
        ));
        match conn.execute(&query, ()) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn select_one_by<T: fmt::Display>(&self, h: &Header<T>) -> Result<Self> {
        let conn = utils::open(Config::database_location())?;
        let query = query_wrapper(format!(
            "SELECT * FROM {} WHERE {}='{}' ORDER BY {} DESC LIMIT 1",
            TABLE_NAME, h.name, h.value, self.id.name
        ));

        let mut stmt = conn.prepare(&query)?;
        let mut user_iter = stmt.query_map([], |row| Self::from_row(row))?;
        user_iter
            .next()
            .unwrap_or_else(|| Err(rusqlite::Error::QueryReturnedNoRows))
    }

    pub fn select_all(&self) -> Result<Vec<User>> {
        let conn = utils::open(Config::database_location())?;
        let query = query_wrapper(format!("SELECT * FROM {}", TABLE_NAME));

        let mut stmt = conn.prepare(&query)?;
        let user_iter = stmt.query_map([], |row| Self::from_row(row))?;
        user_iter.collect()
    }

    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        let mut u = User::new();
        u.id.value = row.get(0)?;
        u.name.value = row.get(1)?;
        u.manager.value = row.get(2)?;
        u.chat_id.value = row.get(3)?;
        Ok(u)
    }
}

impl Default for User {
    fn default() -> Self {
        Self::new()
    }
}

impl From<fsm::Data> for User {
    fn from(data: fsm::Data) -> Self {
        let mut u = User::new();
        u.name.value = data.name;
        u.manager.value = data.manager;
        u.chat_id.value = data.chat_id;
        u
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{
        config::Config,
        db::{user_table::User, utils::DatabaseSource},
    };

    #[test]
    pub fn test_user_insert() {
        if let DatabaseSource::File(path) = Config::database_location() {
            let _ = fs::remove_file(path);
        }

        pretty_env_logger::formatted_timed_builder()
            .filter(Some("guardian"), log::LevelFilter::Trace)
            .init();

        let mut u1 = User::new();
        assert_eq!(Ok(()), u1.create_table());
        let u2 = u1.insert().unwrap();
        assert_eq!(u1.chat_id, u2.chat_id);
        assert_ne!(u1.id, u2.id);

        u1.name.value = "Jonny Black".to_string();
        u1.chat_id.value += 1;
        let u3 = u1.insert_or_update().unwrap();
        assert_eq!(u3.name.value, "Jonny Black".to_string());
        let collection = u1.select_all();
        println!("collection name: {:?}", collection);
        u1.name.value = "Jonny White".to_string();
        let _ = u1.insert_or_update();
        let collection = u1.select_all();
        println!("collection rename: {:?}", collection);

        assert_eq!(collection.unwrap().len(), 2);

        let found = u1.select_one_by(&u1.chat_id);
        println!("found: {:?}", found);
        let found = u1.select_one_by(&u1.name);
        println!("found: {:?}", found);
    }
}
