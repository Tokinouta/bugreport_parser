use regex::Regex;
use std::fs::File;
use std::io::{self, BufWriter, Write};

// 定义 TraceAndFile 结构体
#[derive(Debug)]
struct TraceAndFile {
    traces: Vec<String>,
    log_file_paths: Vec<String>,
}

impl TraceAndFile {
    fn new() -> Self {
        TraceAndFile {
            traces: Vec::new(),
            log_file_paths: Vec::new(),
        }
    }

    fn set_traces(&mut self, traces: Vec<String>) {
        self.traces = traces;
    }

    fn add_log_file_path(&mut self, path: String) {
        self.log_file_paths.push(path);
    }

    fn write_trace_and_log_files(&self, writer: &mut BufWriter<File>) -> io::Result<()> {
        writeln!(writer, " ")?;
        writeln!(
            writer,
            "<<<<<<<<<<<<<<<<<<<<<<<<<<<{} times>>>>>>>>>>>>>>>>>>>>>>>>>>>",
            self.log_file_paths.len()
        )?;

        if self.traces.is_empty() {
            writeln!(writer, "  no trace")?;
        } else {
            for trace in &self.traces {
                writeln!(writer, "{}", trace)?;
            }
        }

        for path in &self.log_file_paths {
            writeln!(writer, "{}", path)?;
        }

        Ok(())
    }
}

// 定义 ANRResultBean 结构体
#[derive(Debug)]
pub struct ANRResultBean {
    process_name: String,
    trace_file_list: Vec<TraceAndFile>,
}

impl ANRResultBean {
    pub fn new() -> Self {
        ANRResultBean {
            process_name: String::new(),
            trace_file_list: Vec::new(),
        }
    }

    pub fn set_process_name(&mut self, name: String) {
        self.process_name = name;
    }

    fn get_process_name(&self) -> &str {
        &self.process_name
    }

    pub fn add_traces(&mut self, traces: &Vec<String>) -> usize {
        let mut trace_and_file = TraceAndFile::new();
        let re = Regex::new(r"\d").unwrap();

        for trace in traces {
            let masked_trace = re.replace_all(&trace, "X").to_string();
            trace_and_file.traces.push(masked_trace);
        }

        self.trace_file_list.push(trace_and_file);
        self.trace_file_list.len() - 1
    }

    fn set_traces(&mut self, traces: &Vec<String>, index: usize) {
        if index >= self.trace_file_list.len() {
            return;
        }

        let re = Regex::new(r"\d").unwrap();
        self.trace_file_list[index].traces.clear();

        for trace in traces {
            let masked_trace = re.replace_all(&trace, "X").to_string();
            self.trace_file_list[index].traces.push(masked_trace);
        }
    }

    pub fn compare_trace(&self, traces: &mut Vec<String>) -> Option<usize> {
        if self.trace_file_list.is_empty() {
            return None;
        }

        let re = Regex::new(r"\d").unwrap();
        let masked_traces: Vec<String> = traces
            .iter()
            .map(|trace| re.replace_all(trace, "X").to_string())
            .collect();

        for (i, trace_and_file) in self.trace_file_list.iter().enumerate() {
            if Self::is_same_list(&masked_traces, &trace_and_file.traces) {
                return Some(i);
            }
        }

        None
    }

    fn is_same_list(list1: &Vec<String>, list2: &Vec<String>) -> bool {
        if list1.is_empty() && list2.is_empty() {
            return true;
        }

        if list1.len() != list2.len() {
            return false;
        }

        for (a, b) in list1.iter().zip(list2.iter()) {
            if a != b {
                return false;
            }
        }

        true
    }

    pub fn add_log_file_path(&mut self, path: String, index: usize) {
        if index < self.trace_file_list.len() {
            self.trace_file_list[index].add_log_file_path(path);
        }
    }

    pub fn write_to_file(&self, writer: &mut BufWriter<File>) -> io::Result<()> {
        writeln!(
            writer,
            "---------------------------begin {}---------------------------",
            self.process_name
        )?;

        for trace_and_file in &self.trace_file_list {
            trace_and_file.write_trace_and_log_files(writer)?;
        }

        writeln!(
            writer,
            "---------------------------end {}---------------------------",
            self.process_name
        )?;

        Ok(())
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_add_traces() {
        // 示例 ANRResultBean
        let mut anr_result = ANRResultBean::new();
        anr_result.set_process_name("example_process".to_string());

        // 添加 traces
        let traces = vec![
            "Thread 1234 waiting".to_string(),
            "Thread 5678 running".to_string(),
        ];
        let index = anr_result.add_traces(&traces);

        // 添加日志文件路径
        anr_result.add_log_file_path("path/to/log1.log".to_string(), index);

        // 写入文件
        if let Ok(file) = File::create("path/to/output_file") {
            let mut writer = BufWriter::new(file);
            if let Err(e) = anr_result.write_to_file(&mut writer) {
                eprintln!("Error writing to file: {}", e);
            }
        }
    }
}
