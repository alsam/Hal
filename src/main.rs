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

use heliocron::{calc, structs, enums};
use chrono::{FixedOffset, Local, TimeZone, Duration};
use serde_json::json;
use std::fs::OpenOptions;
use std::io::{self};
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
    time_offset: i64,

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

fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    if opt.verbose > 2 {
        println!("{:#?}", opt);
        println!("time_offset {}", opt.time_offset);
    }

    let coords = structs::Coordinates {
        latitude: structs::Latitude { value: opt.latitude },
        longitude: structs::Longitude { value: opt.longitude },
    };

    let date = Local::today()
        .and_hms(12, 0, 0)
        .with_timezone(&FixedOffset::from_offset(Local::today().offset()));

    let solar_calculations = calc::SolarCalculations::new(date, coords);
    let calc = |op: &str|
    {
        solar_calculations.calculate_event_time(enums::Event::new(op, None).unwrap())
    };
    let sunrise = calc("sunrise");
    let sunset = calc("sunset");

    let just_time = |ev: &structs::EventTime, offset: &Duration, offset_sign: bool|
    {
        let dt = ev.datetime.unwrap();
        let time = if offset_sign { dt.checked_add_signed(*offset) }
                   else           { dt.checked_sub_signed(*offset) };
        time.unwrap().time()
    };

    let offset_in_minutes = Duration::minutes(opt.time_offset);
    let sunrise_sunset_json = json!({ "day_start" : format!("{}", just_time(&sunrise, &offset_in_minutes, true)),
                                      "day_end"   : format!("{}", just_time(&sunset, &offset_in_minutes, false)) });

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
