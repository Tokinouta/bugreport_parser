use std::fs::File;
use std::io;
use std::path::Path;

use chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone};
use memmap2::Mmap;

use super::dumpsys::Dumpsys;
use super::logcat::{LogcatLine, LogcatSection};
use super::metadata::Metadata;
use super::section::{Section, SectionContent, SECTION_BEGIN, SECTION_BEGIN_NO_CMD, SECTION_END};
#[derive(Debug)]
pub struct Bugreport {
    pub raw_file: Mmap,
    pub metadata: Metadata,
    pub sections: Vec<Section>,
}

impl Bugreport {
    pub fn new(path: &Path) -> io::Result<Self> {
        // Open the file
        let raw_file = File::open(path)?;
        let mmap_file = unsafe { Mmap::map(&raw_file)? };
        Ok(Bugreport {
            raw_file: mmap_file,
            metadata: Metadata::new(),
            sections: Vec::new(),
        })
    }

    pub fn load(&mut self) -> io::Result<()> {
        let result = self.read_and_slice()?;
        self.pair_sections(&result);
        Ok(())
    }

    fn read_and_slice(&mut self) -> io::Result<Vec<(usize, String)>> {
        let bugreport = std::str::from_utf8(&self.raw_file).unwrap();
        let mut lines = bugreport.lines();

        self.metadata.parse(&mut lines)?;

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

        for (line_number, line) in lines.enumerate() {
            let line_number = line_number + self.metadata.lines_passed;
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
        Ok(matches)
    }

    fn pair_sections(&mut self, matches: &Vec<(usize, String)>) {
        // iterate over matches with indices
        let mut second_occurance = false;
        let bugreport = std::str::from_utf8(&self.raw_file).unwrap();
        let lines: Vec<&str> = bugreport.lines().collect();
        for (index, (line_number, content)) in matches.iter().enumerate() {
            if index > 0 && content.contains(&matches.get(index - 1).unwrap().1) {
                second_occurance = true;
            }
            if !second_occurance {
                continue;
            }

            let start_line = matches.get(index - 1).unwrap().0;
            let end_line = *line_number;
            let mut current_section = Section::new(
                content.to_string(),
                start_line + 1,
                end_line - 1,
                match content.as_str() {
                    "SYSTEM LOG" => SectionContent::SystemLog(LogcatSection::new(Vec::new())),
                    "EVENT LOG" => SectionContent::EventLog(LogcatSection::new(Vec::new())),
                    "DUMPSYS" => SectionContent::Dumpsys(Dumpsys::new()),
                    _ => SectionContent::Other,
                },
            );

            current_section.parse(
                &lines[start_line + 1..end_line],
                self.metadata.timestamp.year(),
            );

            self.sections.push(current_section);

            second_occurance = false;
        }
    }

    pub fn get_sections(&self) -> &Vec<Section> {
        &self.sections
    }
}

impl Bugreport {
    pub fn search_by_tag(&self, tag: &str) -> Option<Vec<LogcatLine>> {
        let sections = self
            .sections
            .iter()
            .filter(|s| s.name == "SYSTEM LOG" || s.name == "EVENT LOG")
            .collect::<Vec<&Section>>();
        let mut results = Vec::new();
        for section in sections {
            if let Some(lines) = section.search_by_tag(tag) {
                results.extend(lines);
            }
        }
        Some(results)
    }

    pub fn get_metadata(&self) -> &Metadata {
        &self.metadata
    }
}

pub fn test_setup_bugreport() -> io::Result<Bugreport> {
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
        let mut archive = zip::ZipArchive::new(zip_file)?;

        // 解压整个 ZIP 文件
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_name = file.name();

            let mut file_path = std::path::PathBuf::from("tests/data");
            file_path.push(file_name);
            println!("Extracting {}...", file_path.display());

            // 创建目标文件或目录
            if file.is_dir() {
                std::fs::create_dir_all(file_path)?;
            } else {
                if let Some(parent) = Path::new(&file_path).parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent)?;
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

// TODO: write a bugreport setup function that calls `Bugreport::load()` inside

mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone};

    #[test]
    fn test_read_and_slice() {
        let mut bugreport = test_setup_bugreport().unwrap();
        let matches = bugreport.read_and_slice().unwrap();
        assert_eq!(matches.len(), 274);
        assert_eq!(
            bugreport.metadata.timestamp,
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
        let mut bugreport = test_setup_bugreport().unwrap();

        let start = std::time::Instant::now();
        let matches = match bugreport.read_and_slice() {
            Ok(matches) => matches,
            Err(e) => {
                println!("Error: {}", e);
                return;
            }
        };
        let duration = start.elapsed();
        println!("Time taken: {:?}", duration);
        let start = std::time::Instant::now();
        bugreport.pair_sections(&matches);
        let duration = start.elapsed();
        println!("Time taken: {:?}", duration);
        assert_eq!(bugreport.sections.len(), 134);

        // find the section with the name "SYSTEM LOG"
        let system_log_section = bugreport.sections.iter().find(|s| s.name == "SYSTEM LOG");
        assert_eq!(
            system_log_section.unwrap().content,
            SectionContent::SystemLog(LogcatSection::new(Vec::new()))
        );
        // find the section with the name "EVENT LOG"
        let event_log_section = bugreport.sections.iter().find(|s| s.name == "EVENT LOG");
        assert_eq!(
            event_log_section.unwrap().content,
            SectionContent::EventLog(LogcatSection::new(Vec::new()))
        );
        // find the section with the name "DUMPSYS"
        let dumpsys_section = bugreport.sections.iter().find(|s| s.name == "DUMPSYS");
        assert_eq!(
            dumpsys_section.unwrap().content,
            SectionContent::Dumpsys(Dumpsys::new())
        );
        // find a section without the above names
        let other_section = bugreport
            .sections
            .iter()
            .find(|s| s.name != "SYSTEM LOG" && s.name != "EVENT LOG" && s.name != "DUMPSYS");
        assert_eq!(other_section.unwrap().content, SectionContent::Other);
    }

    #[test]
    fn test_parse_line() {
        let mut bugreport = test_setup_bugreport().unwrap();
        let matches = match bugreport.read_and_slice() {
            Ok(matches) => matches,
            Err(e) => {
                println!("Error: {}", e);
                return;
            }
        };
        bugreport.pair_sections(&matches);

        // find the second section with name "SYSTEM LOG"
        let system_log_sections = bugreport
            .sections
            .iter_mut()
            .filter(|s| s.name == "SYSTEM LOG")
            .collect::<Vec<&mut Section>>();

        let system_log_section_1st = system_log_sections.get(0).unwrap();
        let lines = match &system_log_section_1st.content {
            SectionContent::SystemLog(lines) => lines,
            _ => panic!("Expected SystemLog section type"),
        };

        assert_eq!(lines.len(), system_log_section_1st.get_line_numbers() - 7);
        // The seven lines that cannot be parsed are listed below:
        // These three are as expected:
        // No such line: "--------- beginning of system"
        // No such line: "--------- beginning of crash"
        // No such line: "--------- beginning of main"
        // The following four do not contain a colon (WTF?!!):
        // No such line: "08-16 10:01:26.784  1000  5098  5098 D QSRecord custom(com.google.android.as/com.google.android.apps.miphone.aiai.captions.quicset listening to true"
        // No such line: "08-16 10:01:29.628  1000  5098  5098 D QSRecord custom(com.google.android.as/com.google.android.apps.miphone.aiai.captions.quicset listening to false"
        // No such line: "08-16 10:01:29.976  1000  5098  5098 D QSRecord custom(com.google.android.as/com.google.android.apps.miphone.aiai.captions.quicset listening to true"
        // No such line: "08-16 10:01:31.110  1000  5098  5098 D QSRecord custom(com.google.android.as/com.google.android.apps.miphone.aiai.captions.quicset listening to false"

        let system_log_section_2nd = system_log_sections.get(1).unwrap();
        let lines = match &system_log_section_2nd.content {
            SectionContent::SystemLog(lines) => lines,
            _ => panic!("Expected SystemLog section type"),
        };

        assert_eq!(lines.len(), system_log_section_2nd.get_line_numbers() - 2);
        // The two lines that cannot be parsed are listed below:
        // No such line: "--------- beginning of system"
        // No such line: "--------- beginning of main"

        let event_log_section = bugreport
            .sections
            .iter_mut()
            .find(|s| s.name == "EVENT LOG")
            .unwrap();
        let lines = match &event_log_section.content {
            SectionContent::EventLog(lines) => lines,
            _ => panic!("Expected EventLog section type"),
        };

        assert_eq!(lines.len(), event_log_section.get_line_numbers() - 1);
        // The one line that cannot be parsed is listed below:
        // No such line: "--------- beginning of events"
    }
}
