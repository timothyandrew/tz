use chrono::offset::TimeZone;
use chrono::{DateTime, Datelike, Local};

use chrono_tz::{Tz, TZ_VARIANTS};
use regex::{Captures, Regex};

use std::fs::{self, read_link};
use std::io;
use std::path::Path;
use std::str::FromStr;

/// Given a timezone string (like 'Asia/Kolkata'), return a chrono `Tz` that represents it
pub fn parse_tz(tz: &str) -> Option<Tz> {
    let result = Tz::from_str(tz);

    if let Ok(tz) = result {
        Some(tz)
    } else {
        let tz = tz.to_lowercase();
        TZ_VARIANTS
            .iter()
            .find(|&variant| {
                variant
                    .name()
                    .to_lowercase()
                    .replace("_", " ")
                    .contains(&tz)
            })
            .map(|tz| tz.to_owned())
    }
}

// Given a `Tz`, convert the given date/time string to a DateTime in that timezone
pub fn parse_datetime_in_tz(tz: Tz, datetime: &str) -> Option<DateTime<Tz>> {
    let only_date = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    let only_time = Regex::new(r"^\d{1,2}:\d{2}$").unwrap();
    let date_and_time = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{1,2}:\d{2}$").unwrap();
    let short_time = Regex::new(r"^(\d+):?(\d+)?\s?(am|pm)$").unwrap();

    let datetime = datetime.to_lowercase();

    let datetime = if only_date.is_match(&datetime) {
        format!("{} 00:00", datetime)
    } else if short_time.is_match(&datetime) {
        parse_short_time(short_time.captures(&datetime).unwrap())
    } else if only_time.is_match(&datetime) {
        let today = Local::now();
        format!(
            "{}-{}-{} {}",
            today.year(),
            today.month(),
            today.day(),
            datetime
        )
    } else if date_and_time.is_match(&datetime) {
        datetime.to_owned()
    } else {
        return None;
    };

    let format = "%Y-%m-%d %H:%M";
    tz.datetime_from_str(&datetime, format).ok()
}

fn parse_short_time(short_time: Captures) -> String {
    let hour = short_time.get(1).unwrap().as_str();
    let minute = short_time
        .get(2)
        .map_or(0, |minute| minute.as_str().parse::<i32>().unwrap());
    let ampm = short_time.get(3).unwrap().as_str();
    let today = Local::now();

    let hour = match ampm {
        "am" => hour.parse::<usize>().unwrap(),
        "pm" => hour.parse::<usize>().unwrap() + 12,
        _ => panic!("Shouldn't get here"),
    };

    format!(
        "{}-{:02}-{:02} {:02}:{:02}",
        today.year(),
        today.month(),
        today.day(),
        hour,
        minute
    )
}

pub fn convert<T: TimeZone>(dt: DateTime<Tz>, to_timezone: T) -> DateTime<T> {
    dt.with_timezone(&to_timezone)
}

pub fn current_tz() -> io::Result<Tz> {
    let linux_path = Path::new("/etc/timezone");
    let macos_path = Path::new("/etc/localtime");

    let tz = if linux_path.exists() {
        fs::read_to_string(linux_path)?
    } else if macos_path.exists() {
        let path = read_link(macos_path)?;
        let path = path
            .strip_prefix("/var/db/timezone/zoneinfo/")
            .expect("Failed to strip TZ prefix");
        path.to_str().unwrap().to_owned()
    } else {
        panic!("Failed to read current TZ")
    };

    let tz = tz.parse().expect("Invalid TZ!");

    Ok(tz)
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Local};
    use chrono_tz::{Asia::Kolkata, Europe::London};

    use super::*;

    #[test]
    fn test_parse_tz() {
        assert_eq!(parse_tz("Asia/Kolkata"), Some(Tz::Asia__Kolkata));
        assert_eq!(parse_tz("FooBar"), None);
        assert_eq!(parse_tz("Europe/London"), Some(Tz::Europe__London));
    }

    #[test]
    fn test_convert() {
        let date = London.ymd(2021, 1, 1).and_hms(8, 8, 8);
        let to_date = Kolkata.ymd(2021, 1, 1).and_hms(13, 38, 8);
        assert_eq!(convert(date, Kolkata), to_date);
    }

    // TODO: Fix this so it passes wherever it's run
    #[test]
    fn test_current_tz() {
        assert_eq!(current_tz().unwrap(), Kolkata);
    }

    #[test]
    fn test_parse_datetime_in_tz() {
        assert_eq!(
            parse_datetime_in_tz(Kolkata, "2021-07-09 05:00"),
            Some(Kolkata.ymd(2021, 07, 09).and_hms(5, 0, 0))
        );

        assert_eq!(
            parse_datetime_in_tz(Kolkata, "2021-07-09 5:00"),
            Some(Kolkata.ymd(2021, 07, 09).and_hms(5, 0, 0))
        );

        assert_eq!(
            parse_datetime_in_tz(Kolkata, "2021-07-09"),
            Some(Kolkata.ymd(2021, 07, 09).and_hms(0, 0, 0))
        );

        let today = Local::now();

        assert_eq!(
            parse_datetime_in_tz(Kolkata, "05:00"),
            Some(
                Kolkata
                    .ymd(today.year(), today.month(), today.day())
                    .and_hms(5, 0, 0)
            )
        );

        assert_eq!(
            parse_datetime_in_tz(Kolkata, "3am"),
            Some(
                Kolkata
                    .ymd(today.year(), today.month(), today.day())
                    .and_hms(3, 0, 0)
            )
        );

        assert_eq!(
            parse_datetime_in_tz(Kolkata, "10pm"),
            Some(
                Kolkata
                    .ymd(today.year(), today.month(), today.day())
                    .and_hms(22, 0, 0)
            )
        );

        assert_eq!(
            parse_datetime_in_tz(Kolkata, "5:30pm"),
            Some(
                Kolkata
                    .ymd(today.year(), today.month(), today.day())
                    .and_hms(17, 30, 0)
            )
        );
    }
}
