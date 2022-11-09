use core::fmt;

use frankenstein::InlineKeyboardButton;

use super::{
    fsm::{Event, UserDisplay},
    telapi::{TeleboardInline, TelerowInline},
    telecom::{ReplyEnum, ReplyInline},
};

pub fn reply_help_event() -> ReplyEnum {
    let help_table: Vec<(Event, &str)> = vec![
        (Event::Help, "Prints this help"),
        (Event::Start, "Start of registration"),
        (Event::Survey, "Start servey"),
        (Event::Report, "Generate report"),
        (Event::Rename, "Change name report"),
    ];
    let mut help_text = String::new();
    for (e, s) in help_table {
        help_text.push_str(&format!("/{} - {}\n", e.to_string().to_lowercase(), s));
    }
    make_reply_text(&help_text)
}

pub fn reply_start_event() -> ReplyEnum {
    make_reply_text("Enter your name")
}

pub fn reply_survey_event() -> ReplyEnum {
    make_reply(
        "How are doing you today?",
        Some(&[&[Event::Allright, Event::More]]),
    )
}

pub fn reply_report_period_event() -> ReplyEnum {
    make_reply(
        "What report do you whant",
        Some(&[
            &[Event::ReportOffsetDay, Event::ReportOffsetWeek],
            &[Event::ReportOffsetMonth, Event::Back],
        ]),
    )
}

pub fn reply_report_event() -> ReplyEnum {
    make_reply(
        "What report do you whant",
        Some(&[&[Event::ReportMe, Event::ReportTeam, Event::ReportAll]]),
    )
}

pub fn reply_reg_manager() -> ReplyEnum {
    make_reply(
        "Choose your fighter",
        Some(&[
            &[Event::LMElina],
            &[Event::LMMikhail],
            &[Event::LMOleksandr],
            &[Event::LMVladyslav],
            &[Event::LMYevgen],
        ]),
    )
}
pub fn reply_not_emplemented() -> ReplyEnum {
    make_reply_text("Not implemented, sorry")
}

pub fn make_reply_text(text: &str) -> ReplyEnum {
    make_reply(text, Option::<&[&[Event]]>::None)
}

pub fn make_reply<T: fmt::Display + UserDisplay>(text: &str, slice: Option<&[&[T]]>) -> ReplyEnum {
    if let Some(range) = slice {
        let mut kbd = TeleboardInline::new();
        for row in range {
            let mut kbd_row = TelerowInline::new();
            for cell in *row {
                let btn_text = cell.to_user_string();
                let btn = InlineKeyboardButton::builder()
                    .text(&btn_text)
                    .callback_data(format!("{}", btn_text))
                    .build();
                kbd_row.push(btn);
            }
            kbd.push(kbd_row);
        }
        ReplyEnum::KeyboardInline(ReplyInline::new(text, kbd))
    } else {
        ReplyEnum::Text(text.to_string())
    }
}
