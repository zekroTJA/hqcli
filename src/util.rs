use anyhow::Result;
use chrono::Duration;

pub fn parse_duration<S: AsRef<str>>(dstr: S) -> Result<Duration> {
    let d = parse_duration::parse(dstr.as_ref())?;
    Ok(Duration::from_std(d)?)
}
