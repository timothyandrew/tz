use chrono::offset::TimeZone;
use chrono::{DateTime, NaiveDateTime};

use chrono_tz::{Tz, TZ_VARIANTS};

use std::fs::{self, read_link};
use std::io;
use std::path::Path;
use std::str::FromStr;

pub fn parse_tz(tz: &str) -> Option<Tz> {
    let result = Tz::from_str(tz);

    if let Ok(tz) = result {
        Some(tz)
    } else {
        let tz = tz.to_lowercase();
        TZ_VARIANTS
            .iter()
            .find(|&variant| variant.name().to_lowercase().contains(&tz))
            .map(|tz| tz.to_owned())
    }
}

pub fn parse_datetime_in_tz(tz: Tz, datetime: &str) -> Option<DateTime<Tz>> {
    let format = "%Y-%m-%d %H:%M";
    let start_of_day = format!("{} 00:00", datetime);

    let datetime = if let Ok(_) = NaiveDateTime::parse_from_str(&start_of_day, &format) {
        &start_of_day
    } else {
        datetime
    };

    tz.datetime_from_str(datetime, format).ok()
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
    use chrono_tz::{Asia::Kolkata, Europe::London};

    use super::*;

    #[test]
    fn test_parse_tz() {
        assert_eq!(parse_tz("Asia/Kolkata"), Ok(Tz::Asia__Kolkata));
        assert_eq!(
            parse_tz("FooBar"),
            Err("'FooBar' is not a valid timezone".to_owned())
        );
        assert_eq!(parse_tz("Europe/London"), Ok(Tz::Europe__London));
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
}
