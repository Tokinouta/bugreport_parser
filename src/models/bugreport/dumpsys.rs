use super::section::{Section, SectionContent};
use regex::Regex;

lazy_static::lazy_static!(
    static ref DUMPSYS: Regex= Regex::new(r"-{9} \d\.\d+s was the duration of dumpsys (.*), ending at").unwrap();
);

#[derive(Debug)]
pub struct Dumpsys(Vec<DumpsysEntry>);

#[derive(Debug)]
pub struct DumpsysEntry {
    pub name: String,
    pub data: String,
}

impl Dumpsys {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn parse(&mut self, lines: &[String], _year: i32)  {
        let mut temp = String::new();
        for line in lines {
            if let Some(captures) = DUMPSYS.captures(line) {
                let name = captures.get(1).unwrap().as_str().to_string();
                self.0.push(DumpsysEntry { name, data: temp.clone() });
                temp.clear();
            } else {
                temp.push_str(line);
                temp.push('\n');
            }
        }
    }
}
