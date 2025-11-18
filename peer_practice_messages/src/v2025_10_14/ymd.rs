use chrono::{Datelike, NaiveDate, Weekday};

fn find_nth_weekday(year: i32, month: u32, weekday: Weekday, n: u32) -> Option<NaiveDate> {
    let mut day_of_week_count = 0;

    (1..=31)
        .filter_map(|day| NaiveDate::from_ymd_opt(year, month, day))
        .find(|date| {
            if date.weekday() == weekday {
                day_of_week_count += 1;
                if day_of_week_count == n {
                    return true;
                }
            }
            false
        })
}

pub fn next_second_and_fourth_fridays(start: NaiveDate, count: usize) -> Vec<NaiveDate> {
    let mut res = Vec::new();
    let mut y = start.year();
    let mut m = start.month();
    while res.len() < count {
        if let Some(date2) = find_nth_weekday(y, m, Weekday::Fri, 2) {
            let d2 = date2.day();
            if add_possible_date(start, count, &mut res, y, m, date2, d2) {
                break;
            }
        }
        if let Some(date4) = find_nth_weekday(y, m, Weekday::Fri, 4) {
            let d4 = date4.day();
            if add_possible_date(start, count, &mut res, y, m, date4, d4) {
                break;
            }
        }
        m += 1;
        if m == 13 {
            m = 1;
            y += 1;
        }
    }
    res
}

fn add_possible_date(
    start: NaiveDate,
    count: usize,
    res: &mut Vec<NaiveDate>,
    y: i32,
    m: u32,
    date4: NaiveDate,
    d4: u32,
) -> bool {
    let is_german_christmas_holiday = (m == 12 && d4 >= 25) || (m == 1 && d4 <= 6);

    if !(is_german_christmas_holiday || y == start.year() && m == start.month() && d4 < start.day())
    {
        res.push(date4);
        if res.len() == count {
            return true;
        }
    }
    false
}

pub fn create_date_options() -> Vec<String> {
    next_second_and_fourth_fridays(chrono::Local::now().date_naive(), 5)
        .iter()
        .map(|date| date.format("%Y-%m-%d").to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, NaiveDate, Weekday};

    fn parse_dates() -> (Vec<NaiveDate>, NaiveDate) {
        let today = chrono::Local::now().date_naive();
        let dates: Vec<NaiveDate> = create_date_options()
            .into_iter()
            .map(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").expect("valid date string"))
            .collect();
        (dates, today)
    }

    fn nth_of_month(date: NaiveDate, weekday: Weekday) -> u32 {
        let mut count = 0;
        for day in 1..=date.day() {
            if let Some(d) = NaiveDate::from_ymd_opt(date.year(), date.month(), day)
                && d.weekday() == weekday
            {
                count += 1;
            }
        }
        count
    }

    #[test]
    fn creates_five_formatted_dates() {
        let (dates, _today) = parse_dates();
        assert_eq!(dates.len(), 5, "should return exactly five dates");
    }

    #[test]
    fn returns_next_second_or_fourth_fridays_not_before_today() {
        let (dates, today) = parse_dates();
        for d in &dates {
            assert!(*d >= today, "date {d} should be today or in the future");
            assert_eq!(d.weekday(), Weekday::Fri, "date {d} should be a Friday");
            let nth = nth_of_month(*d, Weekday::Fri);
            assert!(
                nth == 2 || nth == 4,
                "date {d} should be the 2nd or 4th Friday of its month, got #{nth}"
            );
        }
    }

    #[test]
    fn dates_are_strictly_increasing_and_unique() {
        let (dates, _today) = parse_dates();
        for w in dates.windows(2) {
            assert!(
                w[0] < w[1],
                "dates should be strictly increasing: {:?} -> {:?}",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn skips_german_christmas_holidays() {
        let start = NaiveDate::from_ymd_opt(2020, 12, 1).unwrap();
        let dates = next_second_and_fourth_fridays(start, 3);
        assert_eq!(dates.len(), 3, "expected three dates");

        assert!(
            !dates.contains(&NaiveDate::from_ymd_opt(2020, 12, 25).unwrap()),
            "25.12.2020 should be excluded because it's during Christmas/New Year holidays"
        );

        assert_eq!(
            dates[0],
            NaiveDate::from_ymd_opt(2020, 12, 11).unwrap(),
            "expected Dec 11, 2020"
        );
        assert_eq!(
            dates[1],
            NaiveDate::from_ymd_opt(2021, 1, 8).unwrap(),
            "expected Jan 08, 2021"
        );
        assert_eq!(
            dates[2],
            NaiveDate::from_ymd_opt(2021, 1, 22).unwrap(),
            "expected Jan 22, 2021"
        );

        for d in &dates {
            assert_eq!(d.weekday(), Weekday::Fri, "date should be a Friday");
            let nth = nth_of_month(*d, Weekday::Fri);
            assert!(nth == 2 || nth == 4, "date should be the 2nd or 4th Friday");
        }
    }
}
