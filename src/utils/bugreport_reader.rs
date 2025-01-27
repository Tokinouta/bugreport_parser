use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;

use chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref SECTION_BEGIN: Regex = Regex::new(r#"------ (.*?)(?: \((.*)\)) ------"#).unwrap();
    static ref SECTION_BEGIN_NO_CMD: Regex = Regex::new(r#"^------ ([^(]+) ------$"#).unwrap();
    static ref SECTION_END: Regex =
        Regex::new(r#"------ (\d+.\d+)s was the duration of '(.*?)(?: \(.*\))?' ------"#).unwrap();
    static ref LOGCAT_LINE: Regex = Regex::new(
        r#"(\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}) +(\w+) +(\d+) +(\d+) ([A-Z]) ([a-zA-Z_]) *: (.*)"#
    )
    .unwrap();
}

#[derive(Debug)]
struct LogcatLine {
    timestamp: DateTime<Local>,
    user: String,
    pid: u32,
    tid: u32,
    level: char,
    tag: String,
    message: String,
}

impl LogcatLine {
    fn search() {}
}

impl Display for LogcatLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.timestamp, self.pid, self.tid, self.level, self.tag
        )
    }
}

#[derive(Debug)]
enum SectionContent {
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
struct Section {
    name: String,
    start_line: usize,
    end_line: usize,
    content: SectionContent,
}

#[derive(Debug)]
struct Bugreport {
    raw_file: File,
    timestamp: DateTime<Local>,
    sections: Vec<Section>,
}

impl Bugreport {
    fn new(path: &Path) -> io::Result<Self> {
        // Open the file
        let raw_file = File::open(path)?;
        Ok(Bugreport {
            raw_file,
            timestamp: Local::now(),
            sections: Vec::new(),
        })
    }

    pub fn read_and_slice(&mut self) -> io::Result<Vec<(usize, String)>> {
        let mut reader = BufReader::new(&self.raw_file);

        let mut buf = String::new();
        reader.read_line(&mut buf)?;
        buf.clear();
        reader.read_line(&mut buf)?;
        let parse_timestamp = |line: &str| {
            let timestamp_str = line.trim_start_matches("== dumpstate: ").trim();
            NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
                .map(|naive_dt| Local.from_local_datetime(&naive_dt).unwrap())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
        };
        self.timestamp = parse_timestamp(&buf)?;

        // Create a vector to store the line number and content of each match
        let mut matches: Vec<(usize, String)> = Vec::new();
        let filter_and_add =
            |matches: &mut Vec<(usize, String)>, line_number: usize, group: &str| match group {
                "BLOCK STAT" => {}
                l if l.ends_with("PROTO") => {}
                _ => {
                    matches.push((line_number, group.to_string()));
                }
            };

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;

            // Check if the first regex matches and capture groups
            if let Some(caps) = SECTION_END.captures(&line) {
                if let Some(group) = caps.get(2) {
                    // Get the second capture group
                    filter_and_add(&mut matches, line_number + 1, group.as_str());
                }
            }
            // Check for the second regex
            else if let Some(caps) = SECTION_BEGIN.captures(&line) {
                if let Some(group) = caps.get(1) {
                    filter_and_add(&mut matches, line_number + 1, group.as_str());
                }
            }
            // Check for the third regex
            else if let Some(caps) = SECTION_BEGIN_NO_CMD.captures(&line) {
                if let Some(group) = caps.get(1) {
                    filter_and_add(&mut matches, line_number + 1, group.as_str());
                }
            }
        }

        // Output all the matches stored in the variable
        // for (line_number, content) in &matches {
        //     println!("Line {}: {}", line_number, content);
        // }

        Ok(matches)
    }

    pub fn pair_sections(&mut self, matches: &Vec<(usize, String)>) {
        // iterate over matches with indices
        let mut second_occurance = false;
        self.raw_file.seek(SeekFrom::Start(0)).unwrap();
        let reader = BufReader::new(&self.raw_file);
        let lines = reader
            .lines()
            .map(|ss| ss.unwrap())
            .collect::<Vec<String>>();
        for (index, (line_number, content)) in matches.iter().enumerate() {
            if index > 0 && content.contains(&matches.get(index - 1).unwrap().1) {
                second_occurance = true;
            }
            if !second_occurance {
                continue;
            }

            let mut current_section = Section {
                name: content.to_string(),
                // reader: BufReader::new(self.raw_file.try_clone().unwrap()),
                start_line: matches.get(index - 1).unwrap().0 + 1,
                end_line: *line_number - 1,
                content: match content.as_str() {
                    "SYSTEM LOG" => SectionContent::SystemLog(Vec::new()),
                    "EVENT LOG" => SectionContent::EventLog(Vec::new()),
                    "DUMPSYS" => SectionContent::Dumpsys,
                    _ => SectionContent::Other,
                },
            };

            current_section
                .read_lines(&lines[current_section.start_line..current_section.end_line + 1], self.timestamp.year());

            println!("{:?}", current_section);
            self.sections.push(current_section);

            second_occurance = false;
        }
    }
}

impl Section {
    // TODO: add a argument for the year
    fn parse_line(line: &str, year: i32) -> Option<LogcatLine> {
        if let Some(caps) = LOGCAT_LINE.captures(line) {
            let time_str: String = format!("{year}-{}", caps.get(1).unwrap().as_str());
            let logcat_line = LogcatLine {
                timestamp: NaiveDateTime::parse_from_str(
                    time_str.as_str(),
                    "%Y-%m-%d %H:%M:%S.%3f",
                )
                .map(|naive_dt| Local.from_local_datetime(&naive_dt).unwrap())
                .unwrap(),
                user: caps.get(2).unwrap().as_str().to_string(),
                pid: caps.get(3).unwrap().as_str().parse::<u32>().unwrap(),
                tid: caps.get(4).unwrap().as_str().parse::<u32>().unwrap(),
                level: caps.get(5).unwrap().as_str().chars().next().unwrap(),
                tag: caps.get(6).unwrap().as_str().to_string(),
                message: caps.get(7).unwrap().as_str().to_string(),
            };
            println!("{:?}", logcat_line);
            Some(logcat_line)
        } else {
            None
        }
    }

    fn read_lines(&mut self, lines: &[String], year: i32) {
        match self.content {
            SectionContent::SystemLog(ref mut s) | SectionContent::EventLog(ref mut s) => {
                println!("{}", lines.len());
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
}

mod tests {
    use chrono::{NaiveDate, TimeZone};
    use std::{fs, path::PathBuf};
    use zip::ZipArchive;

    use super::*;

    fn setup() -> io::Result<Bugreport> {
        let file_path = Path::new("tests/data/example.txt");
        if !Path::new(file_path).exists() {
            println!(
                "File '{}' does not exist. Extracting from ZIP...",
                file_path.to_str().unwrap()
            );

            // ZIP 文件路径
            let zip_path = Path::new("tests/data/example.zip");

            // 打开 ZIP 文件
            let zip_file = File::open(zip_path)?;
            let mut archive = ZipArchive::new(zip_file)?;

            // 解压整个 ZIP 文件
            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let file_name = file.name();

                let mut file_path = PathBuf::from("tests/data");
                file_path.push(file_name);
                println!("Extracting {}...", file_path.display());

                // 创建目标文件或目录
                if file.is_dir() {
                    fs::create_dir_all(file_path)?;
                } else {
                    if let Some(parent) = Path::new(&file_path).parent() {
                        if !parent.exists() {
                            fs::create_dir_all(parent)?;
                        }
                    }

                    let mut output_file = File::create(file_path)?;
                    io::copy(&mut file, &mut output_file)?;
                }
            }

            println!("Extraction complete.");
        }
        Ok(Bugreport::new(file_path).unwrap())
    }

    #[test]
    fn test_read_and_slice() {
        let mut bugreport = setup().unwrap();
        let matches = bugreport.read_and_slice().unwrap();
        assert_eq!(matches.len(), 274);
        assert_eq!(
            bugreport.timestamp,
            Local
                .from_local_datetime(
                    &NaiveDate::from_ymd_opt(2024, 8, 16)
                        .unwrap()
                        .and_hms_opt(10, 02, 11)
                        .unwrap(),
                )
                .unwrap()
        );
    }

    #[test]
    fn test_pair_sections() {
        let mut bugreport = setup().unwrap();
        let matches = match bugreport.read_and_slice() {
            Ok(matches) => matches,
            Err(e) => {
                println!("Error: {}", e);
                return;
            }
        };
        bugreport.pair_sections(&matches);
        for section in bugreport.sections.iter() {
            println!("{:?}", section);
        }

        assert_eq!(bugreport.sections.len(), 134);

        // find the section with the name "SYSTEM LOG"
        let system_log_section = bugreport.sections.iter().find(|s| s.name == "SYSTEM LOG");
        assert_eq!(
            system_log_section.unwrap().content,
            SectionContent::SystemLog(Vec::new())
        );
        // find the section with the name "EVENT LOG"
        let event_log_section = bugreport.sections.iter().find(|s| s.name == "EVENT LOG");
        assert_eq!(
            event_log_section.unwrap().content,
            SectionContent::EventLog(Vec::new())
        );
        // find the section with the name "DUMPSYS"
        let dumpsys_section = bugreport.sections.iter().find(|s| s.name == "DUMPSYS");
        assert_eq!(dumpsys_section.unwrap().content, SectionContent::Dumpsys);
        // find a section without the above names
        let other_section = bugreport
            .sections
            .iter()
            .find(|s| s.name != "SYSTEM LOG" && s.name != "EVENT LOG" && s.name != "DUMPSYS");
        assert_eq!(other_section.unwrap().content, SectionContent::Other);
    }

    #[test]
    fn test_parse_line() {
        let mut bugreport = setup().unwrap();
        let matches = match bugreport.read_and_slice() {
            Ok(matches) => matches,
            Err(e) => {
                println!("Error: {}", e);
                return;
            }
        };
        bugreport.pair_sections(&matches);

        let system_log_section = bugreport
            .sections
            .iter_mut()
            .find(|s| s.name == "SYSTEM LOG")
            .unwrap();
        let lines = match &system_log_section.content {
            SectionContent::SystemLog(lines) => lines,
            _ => panic!("Expected SystemLog section type"),
        };

        println!("{:?}", system_log_section);
    }
}
// Check if the first regex matches and capture groups
