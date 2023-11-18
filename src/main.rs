mod config;
mod hq;

use anyhow::Result;
use chrono::{Duration, Local, NaiveDateTime, NaiveTime};
use clap::Parser;
use config::Config;
use env_logger::Env;
use hq::HQ;

/// A very hacky CLI tool to log worktime in HelloHQ
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Pass a configuration file from a given path
    #[arg(short, long)]
    config: Option<String>,

    /// The start date and/or time
    #[arg(
        short,
        long,
        long_help = "The start date and/or time. If no date is passed, the current \
                     date is assumed.\n\
                     The format is 'dd.mm.yyyy HH:MM'."
    )]
    start: Option<String>,

    /// The end date and/or time
    #[arg(
        short,
        long,
        long_help = "The end date and/or time. If no date is passed, the current \
                     date is assumed. If this is not passed as well as `time` is not \
                     passed, the current date and time will be used.\n\
                     The format is 'dd.mm.yyyy HH:MM'."
    )]
    end: Option<String>,

    /// The duration worked; overwrites `end` if both passed
    #[arg(
        short,
        long,
        long_help = "The duration worked which will be added to the start time. \
                     This overwrites the `end` time if both are passed. Also, if
                     this is passed, the `pause` time will not be substracted.\n\
                     The format is a combination of different human reabale duration \
                     values, like for example: '8h30m', '9 hours 15 minutes', ..."
    )]
    time: Option<String>,

    /// The duration of your pause time
    #[arg(
        short,
        long,
        long_help = "The duration you have made pauses this day. This will be \
                     substracted from the `end` time. When `time` is passed \
                     directly, this will have no effect.\n\
                     The format is a combination of different human reabale duration \
                     values, like for example: '8h30m', '9 hours 15 minutes', ..."
    )]
    pause: Option<String>,

    /// Skip all confirmation prompts
    #[arg(short, long)]
    yes: bool,

    /// The log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or(&args.log_level)).try_init()?;

    let config = args
        .config
        .map(Config::from_file)
        .unwrap_or_else(Config::init)?;

    let start = args
        .start
        .as_ref()
        .or(config.defaults.as_ref().and_then(|d| d.start_time.as_ref()))
        .map(parse_datetime)
        .transpose()?
        .ok_or_else(|| anyhow::anyhow!("No start time has been specified."))?;

    let time = args.time.as_ref().map(parse_duration).transpose()?;
    let pause = args
        .pause
        .as_ref()
        .or(config.defaults.as_ref().and_then(|d| d.pause.as_ref()))
        .as_ref()
        .map(parse_duration)
        .transpose()?;

    let end = time.map(|t| Ok(start + t)).unwrap_or_else(|| {
        args.end
            .map(parse_datetime)
            .unwrap_or_else(|| Ok(Local::now().naive_local()))
            .map(|e| e - pause.unwrap_or(Duration::seconds(0)))
    })?;

    if !args.yes {
        let dur = end - start;

        let msg = format!(
            "Do you want to log the following work time?\n\
            \n\
            Start:     {}\n\
            End:       {}\n\
            Duration:  {}\n\n",
            start,
            end,
            humantime::format_duration(round_duration(dur.to_std()?))
        );

        match inquire::Confirm::new(&msg).with_default(false).prompt() {
            Err(err) => anyhow::bail!("Failed hooking prompt: {err}"),
            Ok(false) => anyhow::bail!("Abort."),
            _ => {}
        }
    }

    let hq = HQ::new(
        &config.endpoint,
        (&config.session.key, &config.session.value),
    )?;

    hq.log_worktime(start, end)?;

    Ok(())
}

fn parse_datetime<S: AsRef<str>>(dtstr: S) -> Result<NaiveDateTime> {
    let dtstr = dtstr.as_ref();
    Ok(if dtstr.contains(' ') {
        NaiveDateTime::parse_from_str(dtstr, "%d.%m.%Y %H:%M")?
    } else {
        let now = Local::now();
        NaiveDateTime::new(now.date_naive(), NaiveTime::parse_from_str(dtstr, "%H:%M")?)
    })
}

fn parse_duration<S: AsRef<str>>(dstr: S) -> Result<Duration> {
    let d = parse_duration::parse(dstr.as_ref())?;
    Ok(Duration::from_std(d)?)
}

fn round_duration(d: std::time::Duration) -> std::time::Duration {
    std::time::Duration::from_secs((d.as_secs() / 60) * 60)
}
