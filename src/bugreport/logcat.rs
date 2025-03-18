use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use std::fmt::{self, Display, Formatter};
lazy_static! {
    pub(crate) static ref LOGCAT_LINE: Regex = Regex::new(
        r#"(\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}) +(\w+) +(\d+) +(\d+) ([A-Z]) ([^:]+) *:(.*)"#
    )
    .unwrap();
}

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

#[derive(Debug, Clone)]
pub struct LogcatSection(Vec<LogcatLine>);

impl LogcatLine {
    #[rustfmt::skip]
    pub fn new(
        timestamp: DateTime<Local>,
        user: String,
        pid: u32,
        tid: u32,
        level: char,
        tag: String,
        message: String,
    ) -> Self {
        Self { timestamp, user, pid, tid, level, tag, message }
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

impl LogcatSection {
    pub fn new(lines: Vec<LogcatLine>) -> Self {
        Self(lines)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get_line(&self, index: usize) -> Option<&LogcatLine> {
        self.0.get(index)
    }

    pub fn parse(&mut self, lines: &[&str], year: i32) {
        let parsed_lines: Vec<_> = lines
            .par_iter()
            .filter_map(|line| LogcatLine::parse_line(line, year))
            .collect();
        self.0.extend(parsed_lines);
    }

    pub fn search_by_tag(&self, tag: &str) -> Vec<LogcatLine> {
        self.0
            .par_iter()
            .filter(|line| line.tag == tag)
            .cloned()
            .collect()
    }

    pub fn search_by_time(&self, time: &str) -> Vec<LogcatLine> {
        self.0
            .par_iter()
            .filter(|line| {
                let time = NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M:%S")
                    .map(|naive_dt| Local.from_local_datetime(&naive_dt).unwrap())
                    .unwrap();

                line.timestamp - time <= Duration::seconds(1)
                    && line.timestamp - time >= Duration::seconds(-1)
            })
            .cloned()
            .collect()
    }

    pub fn search_by_level(&self, level: char) -> Vec<LogcatLine> {
        self.0
            .par_iter()
            .filter(|line| line.level == level)
            .cloned()
            .collect()
    }

    pub fn search_by_keyword(&self, keyword: &str) -> Vec<LogcatLine> {
        self.0
            .par_iter()
            .filter(|line| line.message.contains(keyword))
            .cloned()
            .collect()
    }
}

mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone};

    fn get_test_lines() -> Vec<&'static str> {
        r#"
08-16 10:01:30.003  1000  5098  5850 D LocalBluetoothAdapter: isSupportBluetoothRestrict = 0
08-16 10:01:31.003 10160  5140  5140 D RecentsImpl: hideNavStubView
08-16 10:01:32.003 10160  5140  5140 D NavStubView_Touch: setKeepHidden    old=false   new=true
08-16 10:01:33.003 10160  5140  5300 D GestureStubView_Touch: setKeepHidden    old=false   new=false
08-16 10:01:34.003  1000  2270  5305 D PerfShielderService: com.android.systemui|StatusBar|171|1389485333739|171|0|1
08-16 10:01:35.003 10160  5140  5300 W GestureStubView: adaptRotation   currentRotation=0   mRotation=0
08-16 10:01:36.003 10160  5140  5300 D GestureStubView: resetRenderProperty: showGestureStub   isLayoutParamChanged=false
08-16 10:01:37.003 10160  5140  5300 D GestureStubView_Touch: disableTouch    old=false   new=false
08-16 10:01:38.003 10160  5140  5300 D GestureStubView: showGestureStub
08-16 10:01:39.003 10160  5140  5300 D GestureStubView_Touch: setKeepHidden    old=false   new=false
"#.trim().lines().collect::<Vec<&str>>()
    }

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

        #[rustfmt::skip]
        assert_eq!(
            format!("{}", logcat_line),
            format!("{} user 1234 5678 I tag: message", timestamp.format("%Y-%m-%d %H:%M:%S.%f"))
        );
    }

    #[test]
    fn test_search_by_tag() {
        let logcat = get_test_lines();
        let mut section = LogcatSection::new(Vec::new());
        section.parse(&logcat, 2024);
        let result = section.search_by_tag("GestureStubView");
        println!("{:?}", result.clone());
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_search_by_time() {
        let logcat = get_test_lines();
        let mut section = LogcatSection::new(Vec::new());
        section.parse(&logcat, 2024);
        let result = section.search_by_time("2024-08-16 10:01:34");
        println!("{:?}", result.clone());
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_search_by_level() {
        let logcat = get_test_lines();
        let mut section = LogcatSection::new(Vec::new());
        section.parse(&logcat, 2024);
        let result = section.search_by_level('D');
        println!("{:?}", result.clone());
        assert_eq!(result.len(), 9);
    }
}
