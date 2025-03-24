use super::{
    dumpsys::Dumpsys,
    logcat::{LogcatLine, LogcatSection},
};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref SECTION_BEGIN: Regex =
        Regex::new(r#"------ (.*?)(?: \((.*)\)) ------"#).unwrap();
    pub static ref SECTION_BEGIN_NO_CMD: Regex = Regex::new(r#"^------ ([^(]+) ------$"#).unwrap();
    pub static ref SECTION_END: Regex =
        Regex::new(r#"------ (\d+.\d+)s was the duration of '(.*?)(?: \(.*\))?' ------"#).unwrap();
}

#[derive(Debug)]
pub enum SectionContent {
    SystemLog(LogcatSection),
    EventLog(LogcatSection),
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
        self.end_line - self.start_line + 1
    }

    pub fn parse(&mut self, lines: &[&str], year: i32) {
        match self.content {
            SectionContent::SystemLog(ref mut s) | SectionContent::EventLog(ref mut s) => {
                s.parse(lines, year);
            }
            SectionContent::Dumpsys(ref mut s) => {
                s.parse(lines, year);
            }
            _ => {}
        };
    }

    pub fn search_by_tag(&self, tag: &str) -> Option<Vec<LogcatLine>> {
        match self.content {
            SectionContent::SystemLog(ref s) | SectionContent::EventLog(ref s) => {
                Some(s.search_by_tag(tag))
            }
            _ => None,
        }
    }

    pub fn search_by_time(&self, time: &str) -> Option<Vec<LogcatLine>> {
        match self.content {
            SectionContent::SystemLog(ref s) | SectionContent::EventLog(ref s) => {
                Some(s.search_by_time(time))
            }
            _ => None,
        }
    }
}

impl std::fmt::Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, start: {}, end: {}", self.name, self.start_line, self.end_line)
    }
}
