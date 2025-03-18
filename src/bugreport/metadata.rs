use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone};
use regex::Regex;
use std::{io, str::Lines};

lazy_static::lazy_static!(
    // Uptime: up 0 weeks, 0 days, 1 hour, 59 minutes,
    static ref UPTIME_REGEX: Regex= Regex::new(r"up (\d+) weeks?, (\d+) days?, (\d+) hours?, (\d+) minutes?").unwrap();
    // Build fingerprint: 'Xiaomi/haotian/haotian:15/AQ3A.240812.002/OS2.0.107.0.VOBCNXM:userdebug/test-keys'
    static ref VERSION_REGEX: Regex= Regex::new(r"Build fingerprint: '(.*)/(.*)/(.*)/(.*)/(.*):(.*)/(.*)'").unwrap();
);

#[derive(Debug)]
pub struct Metadata {
    pub timestamp: DateTime<Local>,
    pub version: String,
    pub uptime: Duration,
    pub lines_passed: usize,
}

impl Metadata {
    pub fn new() -> Self {
        Metadata {
            timestamp: Local::now(),
            version: String::new(),
            uptime: Duration::seconds(0),
            lines_passed: 0usize,
        }
    }

    pub fn parse(&mut self, lines: &mut Lines) -> io::Result<()> {
        loop {
            let line = self.advance_line(lines).unwrap_or("");
            if line.starts_with("== dumpstate: ") {
                self.timestamp = Self::parse_timestamp(&line)?;
            } else if line.starts_with("Build fingerprint:") {
                self.version = Self::parse_version(line)?;
            } else if line.starts_with("Uptime:") {
                self.uptime = Self::parse_uptime(&line)?;
                break; // Stop parsing here, though there are two more unnecessary lines
            }
        }

        Ok(())
    }

    fn advance_line<'a>(&mut self, lines: &'a mut Lines) -> Option<&'a str> {
        self.lines_passed += 1;
        lines.next()
    }

    fn parse_timestamp(line: &str) -> io::Result<DateTime<Local>> {
        let timestamp_str = line.trim_start_matches("== dumpstate: ").trim();
        NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
            .map(|naive_dt| Local.from_local_datetime(&naive_dt).unwrap())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }

    fn parse_version(line: &str) -> io::Result<String> {
        // let version_str = line.trim_start_matches("Build fingerprint: ").trim();
        if let Some(caps) = VERSION_REGEX.captures(line) {
            let version_str = caps.get(5).unwrap().as_str();
            Ok(version_str.to_string())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid version string".to_string(),
            ))
        }
    }

    fn parse_uptime(line: &str) -> io::Result<Duration> {
        let uptime_str = line.trim_start_matches("Uptime: ").trim();
        if let Some(caps) = UPTIME_REGEX.captures(uptime_str) {
            let weeks = caps.get(1).unwrap().as_str().parse().unwrap();
            let days = caps.get(2).unwrap().as_str().parse().unwrap();
            let hours = caps.get(3).unwrap().as_str().parse().unwrap();
            let minutes = caps.get(4).unwrap().as_str().parse().unwrap();
            println!(
                "{} weeks, {} days, {} hours, {} minutes",
                weeks, days, hours, minutes
            );
            // create a duration from the uptime string
            let duration = Duration::weeks(weeks)
                + Duration::days(days)
                + Duration::hours(hours)
                + Duration::minutes(minutes);
            Ok(duration)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid uptime string".to_string(),
            ))
        }
    }
}

mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    #[test]
    fn test_parse_timestamp() {
        let line = "== dumpstate: 2022-03-14 10:00:00";
        let timestamp = Metadata::parse_timestamp(line).unwrap();
        assert_eq!(timestamp.to_rfc3339(), "2022-03-14T10:00:00+08:00");
    }

    #[test]
    fn test_parse_version() {
        let line = "Build fingerprint: 'Xiaomi/haotian/haotian:15/AQ3A.240812.002/OS2.0.107.0.VOBCNXM:userdebug/test-keys'";
        let version = Metadata::parse_version(line).unwrap();
        assert_eq!(version, "OS2.0.107.0.VOBCNXM");
    }

    #[test]
    fn test_parse_uptime() {
        let line = "Uptime: up 0 weeks, 0 days, 1 hour, 59 minutes";
        let uptime = Metadata::parse_uptime(line).unwrap();
        assert_eq!(uptime.num_minutes(), 119);
    }

    #[test]
    fn test_parse() {
        let binding = fs::read_to_string("./tests/data/example.txt").unwrap();
        let mut lines = binding.lines();
        let mut metadata = Metadata::new();
        metadata.parse(&mut lines).unwrap();
        assert_eq!(metadata.timestamp.to_rfc3339(), "2024-08-16T10:02:11+08:00");
        assert_eq!(metadata.version, "V816.0.12.0.UNCMIXM");
        assert_eq!(metadata.uptime.num_minutes(), 32);
        assert_eq!(metadata.lines_passed, 50);
    }
}
