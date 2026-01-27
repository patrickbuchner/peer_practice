use chrono::TimeZone;
#[allow(dead_code)]
pub fn fixed_timestamp() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
}
