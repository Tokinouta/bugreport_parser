use chrono::{DateTime, Local};
use std::fmt::{self, Display, Formatter};

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
