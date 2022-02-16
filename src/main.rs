// The MIT License (MIT)
//
// Copyright (c) 2022 Alexander Samoilov
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE

use heliocron::{calc, config, errors, structs, enums, subcommands};
use chrono::{DateTime, Duration, FixedOffset, Local, TimeZone};
use serde::{Deserialize, Serialize};
use serde_json::{json, Result};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "hal")]
struct Opt {
    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    #[structopt(
        short = "u",
        long = "out",
        help = "Set the output file name, e.g. /tmp/sunrise_sunset.json, if not set, output to stdout"
    )]
    out: Option<String>,

    #[structopt(
        short = "s",
        long = "time-offset",
        help = "Time offset in minutes",
        default_value = "0"
    )]
    time_offset: usize,

    #[structopt(
        short = "l",
        long = "latitude",
        help = "Set the latitude in decimal degrees.",
        requires = "longitude",
        default_value = "40.7128"
    )]
    latitude: f64,

    #[structopt(
        short = "o",
        long = "longitude",
        help = "Set the longitude in decimal degrees.",
        requires = "latitude",
        default_value = "-74.0060"
    )]
    longitude: f64,
}

/*
fn invoke_heliocron_report(
    date: &str,
    timezone: &str,
    latitude: &str,
    longitude: &str,
    verbose: bool,
) -> (String, String) {
    let mut sunrise_sunset = ("".to_string(), "".to_string());
    let report = Command::new("heliocron")
        .arg("--date")
        .arg(&date)
        .arg("--latitude")
        .arg(&latitude)
        .arg("--longitude")
        .arg(&longitude)
        .arg("--time-zone")
        .arg(&timezone)
        .arg("report")
        .output()
        .expect("failed to execute process");

    if verbose {
        println!(
            "heliocron {} {} {} {} {} {} {} {} {}",
            "--date",
            &date,
            "--latitude",
            &latitude,
            "--longitude",
            &longitude,
            "--time-zone",
            &timezone,
            "report"
        );
    }

    if report.status.success() {
        let to_parse = String::from_utf8_lossy(&report.stdout);
        let lines = to_parse.lines();
        for line in lines {
            let extract_time = |s: &str| {
                let vec = s.split_whitespace().collect::<Vec<&str>>();
                // Sunrise is at:            2022-01-22 10:51:47 +03:00
                // 0       1  2              3          4
                let time = String::from(vec[4]);
                time
            };
            if line.starts_with("Sunrise is at:") {
                sunrise_sunset.0 = extract_time(&line);
            }
            if line.starts_with("Sunset is at:") {
                sunrise_sunset.1 = extract_time(&line);
            }
        }
    } else {
        io::stderr().write_all(&report.stderr).unwrap();
    }
    sunrise_sunset
}
*/

fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    if opt.verbose > 2 {
        println!("{:#?}", opt);
        println!("time_offset {}", opt.time_offset);
    }

    let (date, longitude, latitude, timezone): (String, String, String, String);

    let config = config::Config {
        coordinates: structs::Coordinates {
            latitude: structs::Latitude { value: opt.latitude },
            longitude: structs::Longitude { value: opt.longitude },
        },
        date: Local::today()
        .and_hms(12, 0, 0)
        .with_timezone(&FixedOffset::from_offset(Local::today().offset())),
        action: config::Action::Report,
    };

    let solar_calculations = calc::SolarCalculations::new(config.date, config.coordinates);
    let calc = |op: &str|
    {
        solar_calculations.calculate_event_time(enums::Event::new(op, None).unwrap())
    };
    let sunrise = calc("sunrise");
    let sunset = calc("sunset");

    let just_time = |ev: &structs::EventTime| { ev.datetime.unwrap().time() };

    if opt.verbose > 0 {
        println!("sunrise: {} sunset: {}", just_time(&sunrise), just_time(&sunset));
    }

    let sunrise_sunset_json = json!({ "day_start" : format!("{}", just_time(&sunrise)),
                                            "day_end" : format!("{}", just_time(&sunset)) });

    match opt.out {
        None => println!("{:}", sunrise_sunset_json.to_string()),
        Some(oname) => {
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(oname)?;
            serde_json::to_writer(&file, &sunrise_sunset_json)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nyc_sunrise_sunset() {
        // NYC 40.7128° N, 74.0060° W
        /*
        let (sunrise, sunset) = invoke_heliocron_report(
            "2022-01-24",
            "-05:00",   // TZ offset of NYC: GMT-5
            "40.7128N", // latitude of NYC
            "74.0060W", // longitude of NYC
            false,
        ); // be silent
        assert_eq!(sunrise, "07:12:36");
        assert_eq!(sunset, "17:03:42");
        */
    }

    #[test]
    fn test_ok_sunrise_sunset() {
        // Oakland, CA 37.8044° N, 122.2712° W
        /*
        let (sunrise, sunset) = invoke_heliocron_report(
            "2022-01-25",
            "-08:00",    // TZ offset of Oakland, CA: GMT-8
            "37.8044N",  // latitude of Oakland, CA
            "122.2712W", // longitude of Oakland, CA
            false,
        ); // be silent
        assert_eq!(sunrise, "07:18:10");
        assert_eq!(sunset, "17:24:46");
        */
    }
}
