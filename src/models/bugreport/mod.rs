use lazy_static::lazy_static;
use regex::Regex;

pub mod bugreport;
pub mod dumpsys;
pub mod logcat;
pub mod section;

lazy_static! {
    static ref SECTION_BEGIN: Regex = Regex::new(r#"------ (.*?)(?: \((.*)\)) ------"#).unwrap();
    static ref SECTION_BEGIN_NO_CMD: Regex = Regex::new(r#"^------ ([^(]+) ------$"#).unwrap();
    static ref SECTION_END: Regex =
        Regex::new(r#"------ (\d+.\d+)s was the duration of '(.*?)(?: \(.*\))?' ------"#).unwrap();
    static ref LOGCAT_LINE: Regex = Regex::new(
        r#"(\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}) +(\w+) +(\d+) +(\d+) ([A-Z]) ([^:]+) *:(.*)"#
    )
    .unwrap();
}



