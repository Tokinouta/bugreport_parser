use super::{dumpsys::Dumpsys, logcat::LogcatLine};
use chrono::{Duration, Local, NaiveDateTime, TimeZone};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref SECTION_BEGIN: Regex = Regex::new(r#"------ (.*?)(?: \((.*)\)) ------"#).unwrap();
    pub static ref SECTION_BEGIN_NO_CMD: Regex = Regex::new(r#"^------ ([^(]+) ------$"#).unwrap();
    pub static ref SECTION_END: Regex =
        Regex::new(r#"------ (\d+.\d+)s was the duration of '(.*?)(?: \(.*\))?' ------"#).unwrap();
}

#[derive(Debug)]
pub enum SectionContent {
    SystemLog(Vec<LogcatLine>),
    EventLog(Vec<LogcatLine>),
    Dumpsys(Dumpsys),
    Other,
}

impl PartialEq for SectionContent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::SystemLog(_), Self::SystemLog(_)) => true,
            (Self::EventLog(_), Self::EventLog(_)) => true,
            (Self::Dumpsys(_), Self::Dumpsys(_)) => true,
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

    pub fn get_line_numbers(&self) -> usize {
        self.end_line - self.start_line
    }

    pub fn parse(&mut self, lines: &[String], year: i32) {
        println!("Parsing section: {}", lines.len());
        match self.content {
            SectionContent::SystemLog(ref mut s) | SectionContent::EventLog(ref mut s) => {
                // read from start_line to end_line and parse each line
                let mut no_such_line = Vec::new();
                let mut last = 0;
                for (i, line) in lines.into_iter().enumerate() {
                    if let Some(logcat_line) = LogcatLine::parse_line(&line, year) {
                        s.push(logcat_line);
                        if i - last > 1 {
                            no_such_line.push(i - 1);
                            println!("No such line: {:?}", lines[i - 1]);
                        }
                        last = i;
                    };
                }
                println!("No such line: {:?}", no_such_line);
            }
            SectionContent::Dumpsys(ref mut s) => {
                s.parse(lines, year);
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

impl Section {
    // pair input_focus logs within event log
    // 第一步通过 dump of service greezer 找到用户开关屏幕的时间点，也可以考虑通过 screen_toggled 0
    // 第二步根据上述开关屏时间点找当时的 input_focus 记录，看看每一个时间点的 focus 到底在哪里
    // 第三步看 wm 生命周期，看能不能跟 focus 记录对上
    pub fn pair_input_focus(&self, event_log: &[LogcatLine]) -> Vec<(LogcatLine, LogcatLine)> {
        let mut result = Vec::new();
        for line in event_log {
            if line.tag == "InputFocus" {
                result.push(line.clone());
            }
        }
        result
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
        section.parse(&logcat, 2024);
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
        section.parse(&logcat, 2024);
        let result = section.search_by_time("2024-08-16 10:01:34");
        println!("{:?}", result.clone().unwrap());
        assert_eq!(result.unwrap().len(), 2);
    }
}
