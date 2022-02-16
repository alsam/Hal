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

use structopt::StructOpt;
use std::process::Command;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use serde_json::{json, Result};
use std::fs::OpenOptions;
use std::error::Error;
use heliocron::{calc, config, errors, subcommands};

#[derive(StructOpt, Debug)]
#[structopt(name = "hal")]
struct Opt {
    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    #[structopt(short = "u", long = "out", help = "Set the output file name, e.g. /tmp/sunruse_sunset.json, if not set, output to stdout")]
    out: Option<String>,

    #[structopt(short = "d", long = "date")]
    date: Option<String>,

    #[structopt(short = "f", long = "date-format", default_value = "%Y-%m-%d")]
    date_format: String,

    #[structopt(short = "t", long = "time-zone", allow_hyphen_values = true)]
    time_zone: Option<String>,
    #[structopt(
        short = "l",
        long = "latitude",
        help = "Set the latitude in decimal degrees. Can also be set in ~/.config/heliocron.toml.",
        requires = "longitude"
    )]
    latitude: Option<String>,

    #[structopt(
        short = "o",
        long = "longitude",
        help = "Set the longitude in decimal degrees. Can also be set in ~/.config/heliocron.toml.",
        requires = "latitude"
    )]
    longitude: Option<String>,
}

fn invoke_heliocron_report(date: &str,
                           timezone: &str,
                           latitude: &str,
                           longitude: &str,
                           verbose: bool) -> (String, String)
{
    let mut sunrise_sunset = ("".to_string(), "".to_string());
    let report = Command::new("heliocron")
                         .arg("--date")      .arg(&date)
                         .arg("--latitude")  .arg(&latitude)
                         .arg("--longitude") .arg(&longitude)
                         .arg("--time-zone") .arg(&timezone)
                         .arg("report")
                         .output()
                         .expect("failed to execute process");

    if verbose {
        println!("heliocron {} {} {} {} {} {} {} {} {}",
                 "--date", &date, "--latitude", &latitude, "--longitude", &longitude,
                 "--time-zone", &timezone, "report");
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

fn main() -> io::Result<()>
{
    let opt = Opt::from_args();
    if opt.verbose > 2 {
        println!("{:#?}", opt);
    }

    let (date, longitude, latitude, timezone): (String, String, String, String);
    match opt.date {
        None    => date = String::from("2022-01-24"),
        Some(d) => date = d,
    }
    match opt.latitude {
        None    => latitude = String::from("40.7128N"),
        Some(o) => latitude = o,
    }
    match opt.longitude {
        None    => longitude = String::from("74.0060W"),
        Some(t) => longitude = t,
    }
    match opt.time_zone {
        None     => timezone = String::from("-05:00"),
        Some(tz) => timezone = tz,
    }

    let (sunrise, sunset) = invoke_heliocron_report(&date, &timezone, &latitude, &longitude,
                                                    opt.verbose > 0);
    if opt.verbose > 0 {
        println!("sunrise: {} sunset: {}", sunrise, sunset);
    }

    let sunrise_sunset_json = json!({ "day_start" : sunrise, "day_end" : sunset });

    match opt.out {
        None        => println!("{:}", sunrise_sunset_json.to_string()),
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
    fn test_nyc_sunrise_sunset() { // NYC 40.7128° N, 74.0060° W
        let (sunrise, sunset) = invoke_heliocron_report("2022-01-24",
                                                        "-05:00",     // TZ offset of NYC: GMT-5
                                                        "40.7128N",   // latitude of NYC
                                                        "74.0060W",   // longitude of NYC
                                                        false);       // be silent
        assert_eq!(sunrise, "07:12:36");
        assert_eq!(sunset,  "17:03:42");
    }

    #[test]
    fn test_ok_sunrise_sunset() { // Oakland, CA 37.8044° N, 122.2712° W
        let (sunrise, sunset) = invoke_heliocron_report("2022-01-25",
                                                        "-08:00",     // TZ offset of Oakland, CA: GMT-8
                                                        "37.8044N",   // latitude of Oakland, CA
                                                        "122.2712W",  // longitude of Oakland, CA
                                                        false);       // be silent
        assert_eq!(sunrise, "07:18:10");
        assert_eq!(sunset,  "17:24:46");
    }
}
