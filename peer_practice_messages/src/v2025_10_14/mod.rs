use chrono::{Local, NaiveTime, TimeZone, Utc};

pub mod accent_colors;
pub mod authentication;
pub mod email;
pub mod level;
pub mod messages;
pub mod post;
pub mod user;
pub mod ymd;

pub fn convert_to_utc(date: chrono::NaiveDate) -> chrono::DateTime<Utc> {
    let naive_dt = date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    let local_dt = Local
        .from_local_datetime(&naive_dt)
        .single()
        .ok_or("Ambiguous/nonexistent local time")
        .unwrap();
    local_dt.with_timezone(&Utc)
}

pub fn convert_utc_to_local(utc_dt: chrono::DateTime<Utc>) -> chrono::DateTime<Local> {
    utc_dt.with_timezone(&Local)
}

pub fn convert_utc_to_local_date(utc_dt: chrono::DateTime<Utc>) -> chrono::NaiveDate {
    convert_utc_to_local(utc_dt).date_naive()
}
