use chrono::Local;
use chrono::TimeZone;

use chrono_tz::TZ_VARIANTS;
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
// - [x] TARGET_TZ should accept looser input
// - [x] "From" TZ
// - [x] DATETIME should work with just a time
// - [ ] DATETIME should work with things like "5pm"
// - [ ] Convert time ranges

fn main() {
    let matches = App::new("tz")
        .version("0.1")
        .about("Convert between timezones")
        .arg(
            Arg::new("to")
                .short('t')
                .long("to")
                .takes_value(true)
                .required_unless_present_any(&["from"])
                .about("Timezone to convert to (defaults to your current TZ)")
        )
        .arg(
            Arg::new("from")
                .short('f')
                .long("from")
                .takes_value(true)
                .required_unless_present_any(&["to"])
                .about("Timezone to convert from (defaults to your current TZ)")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .takes_value(false)
                .about("Enable verbose output"),
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .takes_value(false)
                .about("List all available timezones"),
        )
        .arg(
            Arg::new("DATETIME")
                .about("Date or time (or both) to convert, defaults to <now> (allowed formats are YYYY-MM-DD, HH:MM, YYYY-MM-DD HH:MM, and HHam/pm)")
                .required(false)
                .index(1),
        )
        .get_matches();

    if matches.occurrences_of("list") == 1 {
        TZ_VARIANTS.iter().for_each(|tz| println!("{}", tz));
        return;
    }

    let verbose = matches.occurrences_of("verbose") == 1;
    let current_tz = current_tz().expect("Failed to determine current timezone");

    let to_tz = matches
        .value_of("to")
        .map(|tz| parse_tz(tz))
        .flatten()
        .unwrap_or(current_tz);

    let from_tz = matches
        .value_of("from")
        .map(|tz| parse_tz(tz))
        .flatten()
        .unwrap_or(current_tz);

    let datetime = matches.value_of("DATETIME");
    let datetime = if let Some(datetime) = datetime {
        parse_datetime_in_tz(from_tz, datetime).expect("Invalid DATETIME")
    } else {
        from_tz
            .from_local_datetime(&Local::now().naive_local())
            .single()
            .expect("Couldn't determine <now>")
    };

    if verbose {
        eprintln!("-> Converting from {} to {}", from_tz, to_tz);
        eprintln!("-> Pre-conversion time: {}\n", datetime);
    }

    let result = convert(datetime, to_tz);

    println!("{}", result);
}
