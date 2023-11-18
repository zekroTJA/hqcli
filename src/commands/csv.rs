use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use super::Command;
use crate::{config::Config, hq::HQ, util};
use anyhow::Result;
use chrono::{Duration, Local, NaiveDate, NaiveDateTime, NaiveTime};
use clap::Args;
use log::info;

/// Takes a CSV of time entries and logs them all in one session
#[derive(Args, Debug)]
pub struct Csv {
    /// The layout and format of the columns
    #[arg(
        short,
        long,
        long_help = "The layout of the values in the data columns and format of the date and time.\n\
                     This can also be specified in the config field `defaults.csv.format`.\n\
                     Please refer this table for all formatting options:\n\
                     https://docs.rs/chrono/latest/chrono/format/strftime/index.html\n\
                     Possible fields are 'date', 'start_time', 'end_time', 'pause' and 'duration'\n\
                     An example could be 'date:%d.%m.%Y,start_time:%H:%M,end_time:%H:%M,pause'"
    )]
    format: Option<String>,

    /// Skip first lines in the CSV file
    #[arg(short, long, default_value_t = 0)]
    skip_lines: usize,

    /// The CSV file
    file: String,
}

enum Field<'a> {
    Date(&'a str),
    StartTime(&'a str),
    EndTime(&'a str),
    Duration,
    Pause,
    Empty,
}

#[derive(Debug)]
struct Entry {
    start: NaiveDateTime,
    end: NaiveDateTime,
}

impl Entry {
    fn read(
        format: &[Field<'_>],
        line: &str,
        default_start_time: Option<NaiveTime>,
    ) -> Result<Self> {
        let split: Vec<&str> = line.split(',').collect();
        if split.len() != format.len() {
            anyhow::bail!("Line column count does not match format column count.");
        }

        let mut date = None;
        let mut start_time = None;
        let mut end_time = None;
        let mut duration = None;
        let mut pause = None;

        for (i, f) in format.iter().enumerate() {
            let v = split[i];

            match f {
                Field::Date(fmt) => date = Some(NaiveDate::parse_from_str(v, fmt)?),
                Field::StartTime(fmt) => start_time = Some(NaiveTime::parse_from_str(v, fmt)?),
                Field::EndTime(fmt) => end_time = Some(NaiveTime::parse_from_str(v, fmt)?),
                Field::Duration => duration = Some(util::parse_duration(v)?),
                Field::Pause => pause = Some(util::parse_duration(v)?),
                Field::Empty => {}
            }
        }

        let date = date.unwrap_or_else(|| Local::now().date_naive());
        let start_time = start_time
            .or(default_start_time)
            .ok_or_else(|| anyhow::anyhow!("No start time has been specified"))?;
        let end_time = end_time
            .map(|et| et - pause.unwrap_or(Duration::seconds(0)))
            .or_else(|| duration.map(|d| start_time + d))
            .ok_or_else(|| anyhow::anyhow!("No end time has been specified"))?;

        Ok(Self {
            start: NaiveDateTime::new(date, start_time),
            end: NaiveDateTime::new(date, end_time),
        })
    }
}

impl Command for Csv {
    fn run(&self, hq: &HQ, config: &Config) -> anyhow::Result<()> {
        let format = self
            .format
            .as_ref()
            .or(config
                .defaults
                .as_ref()
                .and_then(|d| d.csv.as_ref())
                .map(|c| &c.format))
            .ok_or_else(|| anyhow::anyhow!("No format has been specified!"))?;

        info!("Parsing format ...");
        let format = parse_format(format)?;

        info!("Opening CSV file ...");
        let f = File::open(&self.file)?;
        let reader = BufReader::new(f);

        let default_start_time = config
            .defaults
            .as_ref()
            .and_then(|d| d.start_time.as_ref())
            .map(|t| NaiveTime::parse_from_str(t, "%H:%M"))
            .transpose()
            .map_err(|e| anyhow::anyhow!("Failed parsing default start time: {e}"))?;

        info!("Parsing CSV entries ...");
        let mut entries = vec![];
        for line in reader.lines().skip(self.skip_lines) {
            let line = line?;
            let entry = Entry::read(&format, &line, default_start_time)?;
            entries.push(entry);
        }

        info!("Logging working times ...");
        for (i, entry) in entries.iter().enumerate() {
            info!("[{i:>3}] Logging entry ...");
            hq.log_worktime(entry.start, entry.end)?;
        }

        Ok(())
    }
}

fn parse_format(format: &str) -> Result<Vec<Field<'_>>> {
    let format = format.trim();

    if format.is_empty() {
        anyhow::bail!("Format is empty.");
    }

    let split: Vec<&str> = format.split(',').map(str::trim).collect();
    if split.len() < 2 {
        anyhow::bail!("Fields must be separated by commata.")
    }

    split.iter().map(|v| parse_format_field(v)).collect()
}

fn parse_format_field(field: &str) -> Result<Field<'_>> {
    let (name, format) = field
        .split_once(':')
        .map(|(name, format)| (name, Some(format)))
        .unwrap_or((field, None));

    Ok(match name.to_lowercase().as_str() {
        "date" => Field::Date(format.unwrap_or("%d.%m.%Y")),
        "start_time" => Field::StartTime(format.unwrap_or("%H:%M")),
        "end_time" => Field::EndTime(format.unwrap_or("%H:%M")),
        "pause" => Field::Pause,
        "duration" => Field::Duration,
        "" => Field::Empty,
        _ => anyhow::bail!("Invalid format field name '{field}'"),
    })
}
