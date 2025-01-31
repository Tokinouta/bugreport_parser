use super::{logcat::LogcatLine, LOGCAT_LINE};
use chrono::{Duration, Local, NaiveDateTime, TimeZone};

#[derive(Debug)]
pub enum SectionContent {
    SystemLog(Vec<LogcatLine>),
    EventLog(Vec<LogcatLine>),
    Dumpsys,
    Other,
}

impl PartialEq for SectionContent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::SystemLog(_), Self::SystemLog(_)) => true,
            (Self::EventLog(_), Self::EventLog(_)) => true,
            (Self::Dumpsys, Self::Dumpsys) => true,
            (Self::Other, Self::Other) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Section {
    pub name: String,
    start_line: usize,
    end_line: usize,
    pub content: SectionContent,
}

impl Section {
    pub fn new(name: String, start_line: usize, end_line: usize, content: SectionContent) -> Self {
        Self {
            name,
            start_line,
            end_line,
            content,
        }
    }

    fn parse_line(line: &str, year: i32) -> Option<LogcatLine> {
        if let Some(caps) = LOGCAT_LINE.captures(line) {
            let time_str: String = format!("{year}-{}", caps.get(1).unwrap().as_str());
            let logcat_line = LogcatLine::new(
                NaiveDateTime::parse_from_str(time_str.as_str(), "%Y-%m-%d %H:%M:%S.%3f")
                    .map(|naive_dt| Local.from_local_datetime(&naive_dt).unwrap())
                    .unwrap(),
                caps.get(2).unwrap().as_str().to_string(),
                caps.get(3).unwrap().as_str().parse::<u32>().unwrap(),
                caps.get(4).unwrap().as_str().parse::<u32>().unwrap(),
                caps.get(5).unwrap().as_str().chars().next().unwrap(),
                caps.get(6).unwrap().as_str().to_string(),
                caps.get(7).unwrap().as_str().to_string(),
            );
            Some(logcat_line)
        } else {
            None
        }
    }

    pub fn read_lines(&mut self, lines: &[String], year: i32) {
        match self.content {
            SectionContent::SystemLog(ref mut s) | SectionContent::EventLog(ref mut s) => {
                // read from start_line to end_line and parse each line
                for line in lines.into_iter() {
                    if let Some(logcat_line) = Section::parse_line(&line, year) {
                        s.push(logcat_line);
                    };
                }
            }
            _ => {}
        };
    }

    pub fn search_by_tag(&self, tag: &str) -> Option<Vec<LogcatLine>> {
        let content = match self.content {
            SectionContent::SystemLog(ref s) | SectionContent::EventLog(ref s) => s,
            _ => return None,
        };

        let mut result = Vec::new();
        for line in content {
            if line.tag == tag {
                result.push(line.clone());
            }
        }
        Some(result)
    }

    pub fn search_by_time(&self, time: &str) -> Option<Vec<LogcatLine>> {
        let content = match self.content {
            SectionContent::SystemLog(ref s) | SectionContent::EventLog(ref s) => s,
            _ => return None,
        };
        let time = NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M:%S")
            .map(|naive_dt| Local.from_local_datetime(&naive_dt).unwrap())
            .unwrap();

        let mut result = Vec::new();
        for line in content {
            if line.timestamp - time <= Duration::seconds(1)
                && line.timestamp - time >= Duration::seconds(-1)
            {
                result.push(line.clone());
            }
        }
        Some(result)
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_search_by_tag() {
        let logcat = r#"
08-16 10:01:30.003  1000  5098  5850 D LocalBluetoothAdapter: isSupportBluetoothRestrict = 0
08-16 10:01:30.003 10160  5140  5140 D RecentsImpl: hideNavStubView
08-16 10:01:30.003 10160  5140  5140 D NavStubView_Touch: setKeepHidden    old=false   new=true
08-16 10:01:30.003 10160  5140  5300 D GestureStubView_Touch: setKeepHidden    old=false   new=false
08-16 10:01:30.003  1000  2270  5305 D PerfShielderService: com.android.systemui|StatusBar|171|1389485333739|171|0|1
08-16 10:01:30.003 10160  5140  5300 W GestureStubView: adaptRotation   currentRotation=0   mRotation=0
08-16 10:01:30.003 10160  5140  5300 D GestureStubView: resetRenderProperty: showGestureStub   isLayoutParamChanged=false
08-16 10:01:30.003 10160  5140  5300 D GestureStubView_Touch: disableTouch    old=false   new=false
08-16 10:01:30.003 10160  5140  5300 D GestureStubView: showGestureStub
08-16 10:01:30.003 10160  5140  5300 D GestureStubView_Touch: setKeepHidden    old=false   new=false
"#.trim().split("\n").map(|s| s.to_string()).collect::<Vec<String>>();
        let mut section = Section::new(
            "SYSTEM LOG".to_string(),
            0,
            10,
            SectionContent::SystemLog(Vec::new()),
        );
        section.read_lines(&logcat, 2024);
        let result = section.search_by_tag("GestureStubView");
        println!("{:?}", result.clone().unwrap());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_search_by_time() {
        let logcat = r#"
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
"#.trim().split("\n").map(|s| s.to_string()).collect::<Vec<String>>();
        let mut section = Section::new(
            "SYSTEM LOG".to_string(),
            0,
            10,
            SectionContent::SystemLog(Vec::new()),
        );
        section.read_lines(&logcat, 2024);
        let result = section.search_by_time("2024-08-16 10:01:34");
        println!("{:?}", result.clone().unwrap());
        assert_eq!(result.unwrap().len(), 2);
    }
}
