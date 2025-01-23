use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use regex::Regex;

#[derive(Debug)]
struct Section {
    name: String,
    start_line: usize,
    end_line: usize,
}

fn filter_and_add(matches: &mut Vec<(usize, String)>, line_number: usize, group: &str) {
    match group {
        "BLOCK STAT" => {}
        l if l.ends_with("PROTO") => {}
        _ => {
            matches.push((line_number, group.to_string()));
        }
    }
}

pub fn read_and_slice(path: &Path) -> io::Result<Vec<(usize, String)>> {
    let SECTION_BEGIN = Regex::new(r#"------ (.*?)(?: \((.*)\)) ------"#).unwrap();
    let SECTION_BEGIN_NO_CMD = Regex::new(r#"^------ ([^(]+) ------$"#).unwrap();
    let SECTION_END =
        Regex::new(r#"------ (\d+.\d+)s was the duration of '(.*?)(?: \(.*\))?' ------"#).unwrap();

    // Open the file
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Create a vector to store the line number and content of each match
    let mut matches: Vec<(usize, String)> = Vec::new();

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
    for (line_number, content) in &matches {
        println!("Line {}: {}", line_number, content);
    }

    Ok(matches)
}

fn pair_sections(matches: &Vec<(usize, String)>) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    // iterate over matches with indices
    let mut second_occurance = false;
    for (index, (line_number, content)) in matches.iter().enumerate() {
        if index > 0 && content.contains(&matches.get(index - 1).unwrap().1) {
            second_occurance = true;
        }
        if !second_occurance {
            continue;
        }

        let current_section = Section {
            name: content.to_string(),
            start_line: matches.get(index - 1).unwrap().0,
            end_line: *line_number,
        };
        sections.push(current_section);

        second_occurance = false;
    }
    sections
}

mod tests {
    use super::*;
    #[test]
    fn test_read_and_slice() {
        let matches = read_and_slice(Path::new("tests/data/example.txt")).unwrap();
        assert_eq!(matches.len(), 274);
    }

    #[test]
    fn test_pair_sections() {
        let matches = match read_and_slice(Path::new("tests/data/example.txt")) {
            Ok(matches) => matches,
            Err(e) => {
                println!("Error: {}", e);
                return;
            }
        };
        let sections = pair_sections(&matches);
        for section in sections.iter() {
            println!("{:?}", section);
        }
    
        assert_eq!(sections.len(), 134);
    }
}
