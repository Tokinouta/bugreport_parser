use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

use std::fmt::{self, Display, Formatter};

use super::LOGCAT_LINE;

#[derive(Debug, Clone)]
pub struct LogcatLine {
    pub timestamp: DateTime<Local>,
    pub user: String,
    pub pid: u32,
    pub tid: u32,
    pub level: char,
    pub tag: String,
    pub message: String,
}

impl LogcatLine {
    pub fn new(
        timestamp: DateTime<Local>,
        user: String,
        pid: u32,
        tid: u32,
        level: char,
        tag: String,
        message: String,
    ) -> Self {
        Self {
            timestamp,
            user,
            pid,
            tid,
            level,
            tag,
            message,
        }
    }

    pub fn parse_line(line: &str, year: i32) -> Option<Self> {
        if let Some(caps) = LOGCAT_LINE.captures(line) {
            let time_str: String = format!("{year}-{}", caps.get(1).unwrap().as_str());
            let logcat_line = Self::new(
                NaiveDateTime::parse_from_str(time_str.as_str(), "%Y-%m-%d %H:%M:%S.%3f")
                    .map(|naive_dt| Local.from_local_datetime(&naive_dt).unwrap())
                    .unwrap(),
                caps.get(2).unwrap().as_str().to_string(),
                caps.get(3).unwrap().as_str().parse::<u32>().unwrap(),
                caps.get(4).unwrap().as_str().parse::<u32>().unwrap(),
                caps.get(5).unwrap().as_str().chars().next().unwrap(),
                caps.get(6).unwrap().as_str().to_string(),
                caps.get(7).unwrap().as_str().trim().to_string(),
            );
            Some(logcat_line)
        } else {
            None
        }
    }

    pub fn search_by_tag(tag: &str, lines: Vec<LogcatLine>) -> Vec<LogcatLine> {
        lines.into_iter().filter(|line| line.tag.contains(tag)).collect()
    }
}

impl Display for LogcatLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {}: {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S.%f"),
            self.user,
            self.pid,
            self.tid,
            self.level,
            self.tag,
            self.message
        )
    }
}

mod tests {
    use chrono::{NaiveDate, TimeZone};

    use super::*;

    #[test]
    fn test_logcat_line() {
        let timestamp = Local
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2024, 8, 16)
                    .unwrap()
                    .and_hms_opt(10, 02, 11)
                    .unwrap(),
            )
            .unwrap();
        let logcat_line = LogcatLine::new(
            timestamp,
            "user".to_string(),
            1234,
            5678,
            'I',
            "tag".to_string(),
            "message".to_string(),
        );

        assert_eq!(
            format!("{}", logcat_line),
            format!(
                "{} user 1234 5678 I tag: message",
                timestamp.format("%Y-%m-%d %H:%M:%S.%f")
            )
        );
    }
}
