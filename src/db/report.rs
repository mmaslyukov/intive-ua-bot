use chrono::{DateTime, NaiveDateTime, Utc};
use serde::__private::de;
use core::fmt;
use typed_builder::TypedBuilder as Builder;

use crate::{
    bot::{fsm, fsm::Data, Error},
    config::Config,
    db::utils::{self, Header},
};

use super::{
    survey_table::{self, SurveyEntry},
    user_table::{self, User},
    utils::query_wrapper,
};

#[derive(Debug, Builder)]
pub struct Report {
    pub name: Header<String>,
    pub manager: Header<String>,
    pub chat_id: Header<i64>,
    pub timestamp: Header<String>,
    pub electricity: Header<bool>,
    pub network: Header<bool>,
}

impl Default for Report {
    fn default() -> Self {
        Report::new()
    }
}

#[derive(Debug, Clone)]
pub enum TimeOffset {
    Day(i64),
    // Week(i64),
    Month(i64),
}
impl Default for TimeOffset {
    fn default() -> Self {
        TimeOffset::Day(0)
    }
}
impl fmt::Display for TimeOffset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Report {
    pub fn new() -> Self {
        let u = User::new();
        let s = SurveyEntry::new();
        Report::builder()
            .name(Header::new(String::default(), u.name.name))
            .manager(Header::new(String::default(), u.manager.name))
            .chat_id(Header::new(0, u.chat_id.name))
            .timestamp(Header::new(String::default(), s.timestamp.name))
            .electricity(Header::new(true, s.electricity.name))
            .network(Header::new(true, s.network.name))
            .build()
    }

    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        let mut r = Self::new();
        r.name.value = row.get(0)?;
        r.manager.value = row.get(1)?;
        r.chat_id.value = row.get(2)?;
        r.timestamp.value = row.get(3)?;
        r.electricity.value = row.get(4)?;
        r.network.value = row.get(5)?;
        Ok(r)
    }
    pub fn select_all(&self) -> rusqlite::Result<Vec<Self>> {
        Self::select_many(ReportQueryBuilder::new().select().get())
    }

    pub fn select_many_timed(&self, offset: &TimeOffset) -> rusqlite::Result<Vec<Self>> {
        Self::select_many(
            ReportQueryBuilder::new()
                .select()
                .where_()
                .cond_time(offset)
                .get(),
        )
    }

    pub fn select_many_by<T: fmt::Display>(&self, h: &Header<T>) -> rusqlite::Result<Vec<Self>> {
        Self::select_many(
            ReportQueryBuilder::new()
                .select()
                .where_()
                .cond_header(h)
                .get(),
        )
    }

    pub fn select_many_timed_by<T: fmt::Display>(
        &self,
        h: &Header<T>,
        offset: &TimeOffset,
    ) -> rusqlite::Result<Vec<Self>> {
        Self::select_many(
            ReportQueryBuilder::new()
                .select()
                .where_()
                .cond_header(h)
                .and()
                .cond_time(offset)
                .get(),
        )
    }

    fn select_many(query: String) -> rusqlite::Result<Vec<Self>> {
        let conn = utils::open(Config::database_location())?;
        let mut stmt = conn.prepare(&query)?;
        let report_iter = stmt.query_map([], |row| Self::from_row(&row))?;
        report_iter.collect()
    }

    pub fn select_one_by<T: fmt::Display>(&self, h: &Header<T>) -> rusqlite::Result<Self> {
        let conn = utils::open(Config::database_location())?;

        let query = ReportQueryBuilder::new()
            .select()
            .where_()
            .cond_header(h)
            .order()
            .get();

        let mut stmt = conn.prepare(&query)?;
        let mut report_iter = stmt.query_map([], |row| Self::from_row(&row))?;
        report_iter
            .next()
            .unwrap_or_else(|| Err(rusqlite::Error::QueryReturnedNoRows))
    }
}

impl From<fsm::Data> for Report {
    fn from(data: fsm::Data) -> Self {
        let mut r = Report::new();
        if let Some(issues) = data.issues {
            r.network.value = !issues.no_network;
            r.electricity.value = !issues.no_electricity;
        }
        r.name.value = data.name;
        r.chat_id.value = data.chat_id;
        r.timestamp.value = data.utc.format(Config::time_format()).to_string();
        r
    }
}

pub struct ReportQueryBuilder {
    query: String,
}
impl ReportQueryBuilder {
    pub fn new() -> Self {
        Self {
            query: String::default(),
        }
    }
    pub fn select(mut self) -> Self {
        let u = User::new();
        let s = SurveyEntry::new();
        let r = Report::new();
        self.query.push_str(&format!(
            "SELECT {},{},{},{},{},{} FROM {} 
                INNER JOIN {} 
                ON {}.{}={}.{} ",
            r.name.name,
            r.manager.name,
            r.chat_id.name,
            r.timestamp.name,
            r.electricity.name,
            r.network.name,
            user_table::TABLE_NAME,
            survey_table::TABLE_NAME,
            survey_table::TABLE_NAME,
            s.user_id.name,
            user_table::TABLE_NAME,
            u.id.name
        ));
        self
    }
    pub fn where_(mut self) -> Self {
        self.query.push_str(&format!(" WHERE "));
        self
    }
    pub fn and(mut self) -> Self {
        self.query.push_str(&format!(" AND "));
        self
    }

    pub fn cond_header<T: fmt::Display>(mut self, h: &Header<T>) -> Self {
        self.query.push_str(&format!(" {}='{}' ", h.name, h.value));
        self
    }

    pub fn cond_time(mut self, offset: &TimeOffset) -> Self {
        let r = Report::new();
        let format_time = Config::time_format();
        let offset_string = match offset {
            TimeOffset::Day(days) => format!("{} day", days),
            // TimeOffset::Week(week) => format!("{} week", week),
            TimeOffset::Month(month) => format!("{} month", month),
        };

        self.query.push_str(&format!(
            " strftime('{}', {}) >= strftime('{}', '{}', '{}') ",
            format_time,
            r.timestamp.name,
            format_time,
            Utc::now().format(format_time),
            offset_string
        ));
        self
    }
    pub fn order(mut self) -> Self {
        let s = SurveyEntry::new();
        self.query.push_str(&format!(
            " ORDER BY {}.{} DESC LIMIT 1 ",
            survey_table::TABLE_NAME,
            s.id.name,
        ));
        self
    }
    pub fn get(self) -> String {
        query_wrapper(self.query)
    }
}

pub fn survey_save(data: &Data) -> Result<(), Error> {
    let mut result: Result<(), Error> = Ok(());
    let report = Report::from(data.clone());
    let mut insert_survey = false;
    let select_result = report.select_one_by(&report.chat_id);

    match select_result {
        Ok(report) => {
            if let Ok(date_time) =
                NaiveDateTime::parse_from_str(&report.timestamp.value, Config::time_format())
            {
                let utc = DateTime::<Utc>::from_utc(date_time, Utc);
                let delta = chrono::Duration::minutes(10) - Utc::now().signed_duration_since(utc);
                if delta < chrono::Duration::zero() {
                    insert_survey = true;
                } else {
                    result = Error::Verbose(format!(
                        "Too many requests now: {}, last on: {}, plesase wait for {} minute(s) ",
                        Utc::now().format(Config::time_format()),
                        report.timestamp.value,
                        delta.num_minutes() + 1,
                    ))
                    .wrap();
                }
            } else {
                result =
                    Error::Verbose(format!("Can't parse time: {}", report.timestamp.value)).wrap();
            }
        }
        Err(error) => {
            insert_survey = true;
            log::debug!("Survey - true due to no entries: {}", error.to_string());
        }
    }

    log::debug!("insert_survey:{}", insert_survey);
    if insert_survey {
        let mut user = user_table::User::from(data.clone());
        let result = user.select_one_by(&user.chat_id);
        if let Err(_) = result {
            user = user.insert_or_update().unwrap(); // don't care for time being
        } else {
            user = result.unwrap();
        }
        let mut surey: survey_table::SurveyEntry = data.clone().into();
        log::debug!("Data: {:?}", data);
        log::debug!("About to inser survey: {:?}", surey);
        surey.user_id.value = user.id.value;
        surey.insert().unwrap(); // don't care for time being
    }
    result
}

#[cfg(test)]
mod tests {
    use std::fs;

    use chrono::Utc;

    use crate::{
        config::Config,
        db::{report::Report, survey_table::SurveyEntry, user_table::User, utils::DatabaseSource},
    };

    #[test]
    pub fn test_report() {
        if let DatabaseSource::File(path) = Config::database_location() {
            let _ = fs::remove_file(path);
        }

        pretty_env_logger::formatted_timed_builder()
            .filter(Some("guardian"), log::LevelFilter::Trace)
            .init();

        let u1 = User::new();
        assert_eq!(Ok(()), u1.create_table());
        let u2 = u1.insert_or_update().unwrap();
        assert_eq!(u1.chat_id, u2.chat_id);
        assert_ne!(u1.id, u2.id);

        let mut s1 = SurveyEntry::new();
        s1.user_id.value = u2.id.value;
        s1.timestamp.value = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!(Ok(()), s1.create_table());
        let s2 = s1.insert().unwrap();
        assert_ne!(s1.id, s2.id);
        assert_eq!(s1.user_id, s2.user_id);

        let r = Report::new();

        let col = r.select_many_by(&u1.name);
        log::debug!("> {:?}", col);
        let col = r.select_all();
        log::debug!("> {:?}", col);
    }
}
