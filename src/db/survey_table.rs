use core::fmt;

use utils::Header;

use super::{
    user_table,
    utils::{self, query_wrapper},
};
use crate::{bot::fsm, config::Config};
use rusqlite::Result;
use typed_builder::TypedBuilder as Builder;

#[derive(Debug, Builder)]
pub struct SurveyEntry {
    pub id: Header<u64>,
    pub user_id: Header<u64>,
    pub timestamp: Header<String>,
    pub electricity: Header<bool>,
    pub network: Header<bool>,
}

pub const TABLE_NAME: &str = "survey";

impl Default for SurveyEntry {
    fn default() -> Self {
        Self::new()
    }
}

impl SurveyEntry {
    pub fn new() -> Self {
        Self::builder()
            .id(Header::new(0, "id"))
            .user_id(Header::new(0, "user_id"))
            .timestamp(Header::new("".into(), "timestamp"))
            .electricity(Header::new(true, "electricity"))
            .network(Header::new(true, "network"))
            .build()
    }

    pub fn create_table(&self) -> Result<()> {
        let conn = utils::open(Config::database_location())?;
        let query = query_wrapper(format!(
            "CREATE TABLE IF NOT EXISTS {} (
                {}  INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                {}  INTEGER NOT NULL,
                {}  TEXT NOT NULL,
                {}  INTEGER,
                {}  INTEGER,
                FOREIGN KEY ({}) REFERENCES {}({})
                )",
            TABLE_NAME,
            self.id.name,
            self.user_id.name,
            self.timestamp.name,
            self.electricity.name,
            self.network.name,
            self.user_id.name,
            user_table::TABLE_NAME,
            self.id.name, // foreign key. Supposed to use user_table::User::id
        ));
        conn.execute(&query, ())?;

        Ok(())
    }

    pub fn insert(&self) -> Result<Self> {
        let conn = utils::open(Config::database_location())?;
        let query = query_wrapper(format!(
            "INSERT INTO {} ({},{},{},{}) 
                 VALUES (?1, ?2, ?3, ?4)",
            TABLE_NAME,
            self.user_id.name,
            self.timestamp.name,
            self.electricity.name,
            self.network.name
        ));
        conn.execute(
            &query,
            (
                &self.user_id.value,
                &self.timestamp.value,
                &self.electricity.value,
                &self.network.value,
            ),
        )?;

        let query = query_wrapper(format!(
            "SELECT * FROM {} ORDER BY {} DESC LIMIT 1",
            TABLE_NAME, self.id.name
        ));

        let mut stmt = conn.prepare(&query)?;
        let mut user_iter = stmt.query_map([], |row| Self::from_row(row))?;

        user_iter.next().unwrap()
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
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        let mut s = SurveyEntry::new();
        s.id.value = row.get(0)?;
        s.user_id.value = row.get(1)?;
        s.timestamp.value = row.get(2)?;
        s.electricity.value = row.get(3)?;
        s.network.value = row.get(4)?;
        Ok(s)
    }
}

impl From<fsm::Data> for SurveyEntry {
    fn from(data: fsm::Data) -> Self {
        let mut s = SurveyEntry::new();
        if let Some(issues) = data.issues {
            s.network.value = !issues.no_network;
            s.electricity.value = !issues.no_electricity;
        }
        s.timestamp.value = data.utc.format(Config::time_format()).to_string();
        s
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{config::Config, db::utils::DatabaseSource};

    use super::SurveyEntry;

    #[test]
    pub fn test_survey_insert() {
        if let DatabaseSource::File(path) = Config::database_location() {
            let _ = fs::remove_file(path);
        }

        pretty_env_logger::formatted_timed_builder()
            .filter(Some("guardian"), log::LevelFilter::Trace)
            .init();

        let s1 = SurveyEntry::new();
        assert_eq!(Ok(()), s1.create_table());
        let s2 = s1.insert().unwrap();
        assert_ne!(s1.id, s2.id);
        assert_eq!(s1.user_id, s2.user_id);

        // u1.name.value = "Jonny Black".to_string();
        // u1.chat_id.value += 1;
        // let u3 = u1.insert().unwrap();
        // assert_eq!(u3.name.value, "Jonny Black".to_string());
        // let collection = u1.select_all();
        // println!("collection: {:?}", collection);

        // assert_eq!(collection.unwrap().len(), 2);

        // let found = u1.select_by(&u1.chat_id);
        // println!("found: {:?}", found);
        // let found = u1.select_by(&u1.name);
        // println!("found: {:?}", found);
    }
}
