use chrono::Local;
use chrono::TimeZone;

use clap::{App, Arg};
use tz::parse_datetime_in_tz;
use tz::parse_tz;
use tz::{convert, current_tz};

// Command-line API
//
// tz <tz_id> ← Convert current time in current TZ to this TZ
// tz <tz_id> <datetime> ← Convert time in current TZ to this TZ
// tz <tz_id> <datetime> --from <to_tz_id> ← Convert time from the TZ to the to TZ
// tz <tz_id> --from <to_tz_id> ← Convert current time from the from TZ to the to TZ
//
// Timezone IDs can be shortcodes (edt, pst, etc.) or country/city names

// TODO:
// - [x] Basic operation
// - [ ] "From" TZ
// - [ ] DATETIME should work with just a time
// - [ ] DATETIME should work with things like "5pm"
// - [ ] TZ_IDENTIFIER should accept looser input

fn main() {
    let matches = App::new("tz")
        .version("0.1")
        .about("Convert between timezones")
        .arg(
            Arg::new("TZ_IDENTIFIER")
                .about("Timezone to convert to")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .takes_value(false)
                .about("Enable verbose output"),
        )
        .arg(
            Arg::new("DATETIME")
                .about("Date-time to convert (instead of _now_)")
                .required(false)
                .index(2),
        )
        .get_matches();

    let verbose = matches.occurrences_of("verbose") == 1;

    let to_tz = matches.value_of("TZ_IDENTIFIER").unwrap();
    let to_tz = parse_tz(to_tz).expect("Invalid TZ!");
    let from_tz = current_tz().expect("Failed to determine current timezone");

    if verbose {
        eprintln!("-> Detected current location: {}", from_tz);
    }

    let datetime = matches.value_of("DATETIME");
    let datetime = if let Some(datetime) = datetime {
        parse_datetime_in_tz(from_tz, datetime).expect("Invalid DATETIME")
    } else {
        from_tz
            .from_local_datetime(&Local::now().naive_local())
            .single()
            .expect("Couldn't determine <now>")
    };

    let result = convert(datetime, to_tz);

    println!("{}", result);
}
