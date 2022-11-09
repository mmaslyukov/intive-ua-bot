use chrono::{DateTime, NaiveDateTime, Utc};
use typed_builder::TypedBuilder as Builder;

use core::fmt;
use std::{
    collections::HashMap,
    str::{self, FromStr},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    config::Config,
    db::{report, user_table},
};

use super::{telecom::ReplyEnum, utils, Error};
use prettytable::{format, row, table};

#[derive(Debug, EnumIter, PartialEq, Hash, Eq, Clone, Default)]
pub enum Event {
    #[default]
    None,
    Help,
    Report,
    Rename,
    Start,
    Survey,
    Back,
    Name(String),
    Allright,
    More,
    NoNetwork,
    NoElectricity,
    FullBlackout,
    Impact(u8),
    ReportMe,
    ReportTeam,
    ReportAll,
    ReportOffsetDay,
    ReportOffsetWeek,
    ReportOffsetMonth,
    LMMikhail,
    LMElina,
    LMOleksandr,
    LMVladyslav,
    LMYevgen,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
pub trait UserDisplay {
    fn to_user_string(&self) -> String;
}

impl UserDisplay for Event {
    fn to_user_string(&self) -> String {
        match self {
            Self::ReportOffsetDay => "Day".into(),
            Self::ReportOffsetWeek => "Week".into(),
            Self::ReportOffsetMonth => "Month".into(),
            Self::ReportMe => "Me".into(),
            Self::ReportTeam => "My Team".into(),
            Self::ReportAll => "All".into(),
            Self::NoNetwork => "No Network".into(),
            Self::NoElectricity => "No Electricity".into(),
            Self::FullBlackout => "Full Blackout".into(),
            Self::LMElina => "Elina Bodzhek".into(),
            Self::LMMikhail => "Mikhail Maslyukov".into(),
            Self::LMOleksandr => "Oleksandr Ovadenko".into(),
            Self::LMVladyslav => "Vladyslav Symonenko".into(),
            Self::LMYevgen => "Yevgen Kutsenko".into(),

            _ => self.to_string(),
        }
    }
}
impl Event {
    pub fn from_string(text: &str) -> Option<Event> {
        for it in Self::iter() {
            if text
                .to_lowercase()
                .contains(it.to_user_string().to_lowercase().as_str())
            {
                return Some(it.into());
            }
        }
        None
    }
}

#[derive(Debug, EnumIter, Clone)]
pub enum State {
    New(Data),
    Idle(Data),
    RegName(Data),
    RegManager(Data),
    SurvEntry(Data),
    SurvMore(Data),
    Report(Data),
    ReportFrame(Data),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Default, Builder, Clone)]
pub struct Data {
    pub chat_id: i64,
    #[builder(setter(into), default)]
    pub name: String,
    #[builder(setter(into), default)]
    pub manager: String,
    #[builder(setter(into), default)]
    pub utc: DateTime<Utc>,
    #[builder(setter(into, strip_option), default)]
    pub issues: Option<Issues>,
    #[builder(setter(into, strip_option), default)]
    pub reply: Option<ReplyEnum>,
    #[builder(setter(into, strip_option), default)]
    pub report: Option<ReportData>,
}

impl From<report::Report> for Data {
    fn from(item: report::Report) -> Self {
        let mut d = Data::default();
        d.chat_id = item.chat_id.value;
        d.name = item.name.value;
        d.manager = item.manager.value;
        if let Ok(date_time) =
            NaiveDateTime::parse_from_str(&item.timestamp.value, Config::time_format())
        {
            d.utc = DateTime::<Utc>::from_utc(date_time, Utc);
        }
        d
    }
}

impl Data {
    fn on_start(&mut self, _e: Event) -> Result<(), Error> {
        self.reply = Some(utils::reply_start_event());
        Ok(())
    }
    fn on_help(&mut self, _e: Event) -> Result<(), Error> {
        self.reply = Some(utils::reply_help_event());
        Ok(())
    }
    fn on_survey(&mut self, _e: Event) -> Result<(), Error> {
        self.reply = Some(utils::reply_survey_event());
        Ok(())
    }

    // fn not_implemented(&mut self, _e: Event) -> Result<(), Error> {
    //     self.reply = Some(utils::reply_not_emplemented());
    //     Ok(())
    // }
    fn on_reg_name(&mut self, e: Event) -> Result<(), Error> {
        if let Event::Name(name) = e {
            if name
                .chars()
                .all(|i| i.is_alphanumeric() || i.is_whitespace())
            {
                self.name = name.trim().to_string();
                self.reply = Some(utils::reply_reg_manager());
                Ok(())
            } else {
                let error = Error::Verbose(format!("Only characters allowed"));
                self.reply = Some(utils::make_reply_text(error.msg().unwrap().as_str()));
                error.wrap()
            }
        } else {
            Error::Verbose(format!("Unexpected event: {}", e)).wrap()
        }
    }
    fn on_reg_manager(&mut self, e: Event) -> Result<(), Error> {
        self.manager = e.to_user_string();
        let user = user_table::User::from(self.clone());
        match user.insert_or_update() {
            Ok(_) => {
                self.reply = Some(utils::reply_survey_event());
                Ok(())
            }
            Err(e) => {
                let error = Error::Verbose(format!("Can't save: {}", e));
                self.reply = Some(utils::make_reply_text(error.msg().unwrap().as_str()));
                error.wrap()
            }
        }
    }
    fn on_survey_allright(&mut self, _e: Event) -> Result<(), Error> {
        self.issues = None;
        if let Err(e) = report::survey_save(&self) {
            log::debug!("Alright: {}", e.to_string());
            self.reply = Some(utils::make_reply_text(
                format!("{}", e.to_string()).as_str(),
            ));
        } else {
            self.reply = Some(utils::make_reply_text("Take care"));
        }
        Ok(())
    }
    fn on_survey_more(&mut self, _e: Event) -> Result<(), Error> {
        self.reply = Some(utils::make_reply(
            "What's up?",
            Some(&[
                &[Event::NoNetwork, Event::NoElectricity],
                &[Event::FullBlackout, Event::Back],
            ]),
        ));
        Ok(())
    }

    fn on_survey_issue(&mut self, e: Event) -> Result<(), Error> {
        let mut result: Result<(), Error> = Ok(());
        if self.issues.is_none() {
            self.issues = Some(Issues::default());
        }

        match e {
            Event::NoNetwork => self.issues.as_mut().unwrap().no_network = true,
            Event::NoElectricity => self.issues.as_mut().unwrap().no_electricity = true,
            Event::FullBlackout => {
                self.issues.as_mut().unwrap().no_network = true;
                self.issues.as_mut().unwrap().no_electricity = true;
            }
            _ => result = Error::Verbose(format!("Unexpected event: {}", e)).wrap(),
        }

        if let Err(e) = report::survey_save(&self) {
            self.reply = Some(utils::make_reply_text(
                format!("{}", e.to_string()).as_str(),
            ));
        } else {
            self.reply = Some(utils::make_reply_text("Take care"));
        }

        result
    }
    fn on_report(&mut self, _e: Event) -> Result<(), Error> {
        self.reply = Some(utils::reply_report_event());
        Ok(())
    }
    fn on_report_offset(&mut self, e: Event) -> Result<(), Error> {
        let mut result: Result<(), Error> = Ok(());
        if self.report.is_none() {
            self.report = Some(ReportData::default());
        }
        match e {
            Event::ReportOffsetDay => {
                self.report.as_mut().unwrap().offset = report::TimeOffset::Day(-1)
            }
            Event::ReportOffsetWeek => {
                self.report.as_mut().unwrap().offset = report::TimeOffset::Day(-7)
            }
            Event::ReportOffsetMonth => {
                self.report.as_mut().unwrap().offset = report::TimeOffset::Month(-1)
            }
            _ => result = Error::Verbose(format!("Unexpected event: {}", e)).wrap(),
        }
        let report_type = self.report.as_ref().unwrap().report_type.clone();
        let offset = self.report.as_ref().unwrap().offset.clone();
        let summary = make_report(report_type, offset);
        let mut text = String::default();
        let mut period: report::TimeOffset = report::TimeOffset::default();
        let mut table = table!();
        if summary.is_ok() {
            table.set_titles(row!["#", "Full Name", "Availability", "Updated"]);
            table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

            if summary.as_ref().unwrap().len() > 0 {
                period = summary.as_ref().unwrap().first().unwrap().period.clone();
            }
            let mut idx = 0;
            for s in summary.unwrap() {
                idx += 1;
                let days = (Utc::now() - s.last_update).num_days();
                table.add_row(row![
                    format!("{idx}"),
                    format!("{}", s.name),
                    format!("{:.1} %", s.availability * 100.0),
                    format!(
                        "{}",
                        if days != 0 {
                            format!("{} days ago", days)
                        } else {
                            "Today".into()
                        }
                    )
                ]);

                // text.push_str(
                //     format!(
                //         "{}. {}, available:{:.1}%, last update:{} \n",
                //         idx,
                //         s.name,
                //         s.availability * 100.0,
                //         if days != 0 {
                //             format!("{} days ago", days)
                //         } else {
                //             "Today".into()
                //         }
                //     )
                //     .as_str(),
                // );
            }
        }

        self.reply = Some(utils::make_reply_text(
            format!("<pre>Report [{}]:\n{}\n</pre>", period, table.to_string()).as_str(),
        ));
        // log::debug!("Report [{}]:\n{}", period, text);

        result
    }

    fn on_report_type(&mut self, e: Event) -> Result<(), Error> {
        let mut result: Result<(), Error> = Ok(());
        if self.report.is_none() {
            self.report = Some(ReportData::new());
        }
        match e {
            Event::ReportMe => {
                self.report.as_mut().unwrap().report_type = ReportType::Me(self.chat_id)
            }
            Event::ReportTeam => {
                self.report.as_mut().unwrap().report_type = ReportType::Team(self.name.to_owned())
            }
            Event::ReportAll => self.report.as_mut().unwrap().report_type = ReportType::All,
            _ => result = Error::Verbose(format!("Unexpected event: {}", e)).wrap(),
        }

        self.reply = Some(utils::reply_report_period_event());
        result
    }

    pub fn wrap<S>(mut self, state: S) -> State
    where
        S: 'static + Fn(Self) -> State,
    {
        self.update_timestamp();
        let new_state = state(self);
        log::debug!("</TR> transit to {}", new_state.to_string());
        new_state
    }

    fn update_timestamp(&mut self) {
        self.utc = Utc::now();
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, Default)]
pub struct Issues {
    pub no_network: bool,
    pub no_electricity: bool,
    pub impcat: u8,
}

#[derive(Debug, Clone)]
pub struct ReportData {
    pub report_type: ReportType,
    pub offset: report::TimeOffset,
}

impl ReportData {
    fn new() -> Self {
        Self {
            report_type: ReportType::All,
            offset: report::TimeOffset::Day(1),
        }
    }
}
impl Default for ReportData {
    fn default() -> Self {
        Self::new()
    }
}
impl State {
    pub fn consume(mut self, e: Event) -> Transition {
        self.data_mut().reply = None;
        match (self, e) {
            (State::New(data), e @ Event::Start) => {
                Transition::make_valid(State::New, State::RegName, data, e, Data::on_start)
            }
            (State::Idle(data), e @ (Event::Start | Event::Survey)) => {
                Transition::make_valid(State::Idle, State::SurvEntry, data, e, Data::on_survey)
            }
            (State::Idle(data), e @ Event::Report) => {
                Transition::make_valid(State::Idle, State::Report, data, e, Data::on_report)
            }
            (State::RegName(data), e @ Event::Start) => {
                Transition::make_valid(State::RegName, State::RegName, data, e, Data::on_start)
            }
            (State::RegName(data), e @ Event::Name(_)) => Transition::make_valid(
                State::RegName,
                State::RegManager,
                data,
                e,
                Data::on_reg_name,
            ),
            (
                State::RegManager(data),
                e @ (Event::LMElina
                | Event::LMMikhail
                | Event::LMOleksandr
                | Event::LMVladyslav
                | Event::LMYevgen),
            ) => Transition::make_valid(
                State::RegManager,
                State::SurvEntry,
                data,
                e,
                Data::on_reg_manager,
            ),
            (State::RegManager(data), e @ Event::Start) => {
                Transition::make_valid(State::RegManager, State::RegName, data, e, Data::on_start)
            }
            (State::SurvEntry(data), e @ (Event::Start | Event::Survey)) => {
                Transition::make_valid(State::SurvEntry, State::SurvEntry, data, e, Data::on_survey)
            }
            (State::SurvEntry(data), e @ Event::Allright) => Transition::make_valid(
                State::SurvEntry,
                State::Idle,
                data,
                e,
                Data::on_survey_allright,
            ),
            (State::SurvEntry(data), e @ Event::More) => Transition::make_valid(
                State::SurvEntry,
                State::SurvMore,
                data,
                e,
                Data::on_survey_more,
            ),
            (State::SurvEntry(data), e @ Event::Report) => {
                Transition::make_valid(State::SurvEntry, State::Report, data, e, Data::on_report)
            }
            (State::SurvMore(data), e @ (Event::Start | Event::Survey)) => {
                Transition::make_valid(State::SurvMore, State::SurvEntry, data, e, Data::on_survey)
            }
            (
                State::SurvMore(data),
                e @ (Event::NoElectricity | Event::NoNetwork | Event::FullBlackout),
            ) => {
                Transition::make_valid(State::SurvMore, State::Idle, data, e, Data::on_survey_issue)
            }
            (State::SurvMore(data), e @ Event::Back) => {
                Transition::make_valid(State::SurvMore, State::SurvEntry, data, e, Data::on_survey)
            }
            (State::SurvMore(data), e @ Event::Report) => {
                Transition::make_valid(State::SurvMore, State::Report, data, e, Data::on_report)
            }
            (State::Report(data), e @ (Event::ReportMe | Event::ReportTeam | Event::ReportAll)) => {
                Transition::make_valid(
                    State::Report,
                    State::ReportFrame,
                    data,
                    e,
                    Data::on_report_type,
                )
            }
            (State::Report(data), e @ Event::Survey) => {
                Transition::make_valid(State::Report, State::SurvEntry, data, e, Data::on_survey)
            }
            (State::ReportFrame(data), e @ Event::Back) => {
                Transition::make_valid(State::ReportFrame, State::Report, data, e, Data::on_report)
            }
            (
                State::ReportFrame(data),
                e @ (Event::ReportOffsetDay | Event::ReportOffsetWeek | Event::ReportOffsetMonth),
            ) => Transition::make_valid(
                State::ReportFrame,
                State::Idle,
                data,
                e,
                Data::on_report_offset,
            ),
            (State::ReportFrame(data), e @ Event::Survey) => Transition::make_valid(
                State::ReportFrame,
                State::SurvEntry,
                data,
                e,
                Data::on_survey,
            ),
            (s, e @ Event::Help) => Transition::make_general(s, e, Data::on_help),
            (s, e @ Event::Rename) => Transition::make_valid(
                State::Idle,
                State::RegName,
                s.data().clone(),
                e,
                Data::on_start,
            ),

            (s, e) => Transition::make_shallow(s, e),
        }
    }
    pub fn consume_as_str(self, text: &str) -> Transition {
        match self {
            Self::RegName(_) => {
                let event = Event::Name(String::from_str(text).unwrap());
                self.consume(event)
            }
            _ => {
                if let Some(event) = Event::from_string(text) {
                    self.consume(event)
                } else {
                    self.consume(Event::None)
                }
            }
        }
    }

    pub fn data_mut(&mut self) -> &mut Data {
        match self {
            Self::New(data) => data,
            Self::Idle(data) => data,
            Self::RegName(data) => data,
            Self::RegManager(data) => data,
            Self::SurvEntry(data) => data,
            Self::SurvMore(data) => data,
            Self::Report(data) => data,
            Self::ReportFrame(data) => data,
        }
    }

    pub fn data(&self) -> &Data {
        match self {
            Self::New(data) => data,
            Self::Idle(data) => data,
            Self::RegName(data) => data,
            Self::RegManager(data) => data,
            Self::SurvEntry(data) => data,
            Self::SurvMore(data) => data,
            Self::Report(data) => data,
            Self::ReportFrame(data) => data,
        }
    }

    pub fn reply(&self) -> Option<ReplyEnum> {
        self.data().reply.clone()
    }
}

type Wrapper = fn(Data) -> State;
type Handler = fn(&mut Data, Event) -> Result<(), Error>;

pub struct Transition {
    wrap: Option<Wrapper>,
    wrap_fallback: Option<Wrapper>,
    data: Option<Data>,
    next: Option<State>,
    event: Event,
    func: Option<Handler>,
}

impl Transition {
    pub fn make_valid(
        wrap_fallback: Wrapper,
        wrap: Wrapper,
        data: Data,
        event: Event,
        func: Handler,
    ) -> Self {
        Self {
            wrap: Some(wrap),
            wrap_fallback: Some(wrap_fallback),
            data: Some(data),
            next: None,
            event,
            func: Some(func),
        }
    }
    pub fn make_general(next: State, event: Event, func: Handler) -> Self {
        Self {
            wrap: None,
            wrap_fallback: None,
            data: None,
            next: Some(next),
            event,
            func: Some(func),
        }
    }
    pub fn make_shallow(next: State, event: Event) -> Self {
        Self {
            wrap: None,
            wrap_fallback: None,
            data: None,
            next: Some(next),
            event,
            func: None,
        }
    }

    pub fn transit(self) -> State {
        log::debug!("<TR> event: {}", self.event);
        if self.data.is_some() && self.wrap.is_some() && self.func.is_some() {
            let mut data = self.data.unwrap();
            match (self.func.unwrap())(&mut data, self.event) {
                Ok(_) => data.wrap(self.wrap.unwrap()),
                Err(_) => data.wrap(self.wrap_fallback.unwrap()),
            }
        } else {
            let mut state = self.next.unwrap();
            if self.func.is_some() {
                // let mut data = state.data().clone();
                let data = state.data_mut();
                let _ = (self.func.unwrap())(data, self.event);
            }
            state
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReportType {
    Me(i64),
    Team(String),
    All,
}

pub struct ReportSummary {
    name: String,
    manager: String,
    period: report::TimeOffset,
    availability: f64,
    last_update: DateTime<Utc>,
}
impl ReportSummary {
    fn new() -> Self {
        Self {
            name: String::default(),
            manager: String::default(),
            period: report::TimeOffset::Day(-1),
            availability: 0.0,
            last_update: Utc::now(),
        }
    }
}
impl From<&Vec<report::Report>> for ReportSummary {
    fn from(collection: &Vec<report::Report>) -> Self {
        let mut last_update = DateTime::<Utc>::from_utc(
            NaiveDateTime::parse_from_str("1970-01-01 00:00:00", Config::time_format()).unwrap(),
            Utc,
        );
        let mut availability = 0.0;
        collection.iter().for_each(|i| {
            if i.network.value && i.electricity.value {
                availability += 1.0;
            } else if i.network.value {
                availability += 0.5;
            } else {
            }

            if let Ok(date_time) =
                NaiveDateTime::parse_from_str(&i.timestamp.value, Config::time_format())
            {
                let utc = DateTime::<Utc>::from_utc(date_time, Utc);
                if utc > last_update {
                    last_update = utc;
                }
            }
        });
        availability /= collection.len() as f64;
        let mut summary = ReportSummary::new();
        if !collection.is_empty() {
            summary.name = collection.first().unwrap().name.value.clone();
            summary.manager = collection.first().unwrap().manager.value.clone();
            summary.availability = availability;
            summary.last_update = last_update;
        }
        summary
    }
}

fn make_report(
    report_type: ReportType,
    report_period: report::TimeOffset,
) -> Result<Vec<ReportSummary>, Error> {
    let mut report_summary: Vec<ReportSummary> = Vec::new();
    let r = report::Report::new();
    let dataset = match report_type {
        ReportType::All => r.select_many_timed(&report_period),
        ReportType::Me(chat_id) => {
            let mut u = user_table::User::new();
            u.chat_id.value = chat_id;
            r.select_many_timed_by(&u.chat_id, &report_period)
        }
        ReportType::Team(manager_name) => {
            let mut u = user_table::User::new();
            u.manager.value = manager_name;
            r.select_many_timed_by(&u.manager, &report_period)
        }
    };
    if dataset.is_ok() {
        let mut map: HashMap<String, Vec<report::Report>> = HashMap::new();
        for d in dataset.unwrap() {
            let entry = map.entry(d.name.value.clone()).or_insert(Vec::new());
            entry.push(d);
        }
        for k in map.keys() {
            let mut report_one: ReportSummary = map.get(k).unwrap().into();
            report_one.period = report_period.clone();
            report_summary.push(report_one)
        }
    }
    Ok(report_summary)
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_fsm_alt() {}
}
