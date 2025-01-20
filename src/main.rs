use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use models::result_item_bean::ResultItemBean;

pub mod models;
mod trace_analysis;

fn main() {
    let args: Vec<String> = env::args().collect();

    // 检查是否提供了文件名
    if args.len() < 2 {
        eprintln!("error: please input the file name.");
        return;
    }

    let type_arg = &args[1];
    let src_path = if type_arg.len() == 2 {
        if type_arg == "-h" {
            print_help();
            return;
        }
        if args.len() < 3 {
            eprintln!("error: please input the file name.");
            return;
        }
        &args[2]
    } else {
        &args[1]
    };

    // 检查文件路径是否为空
    if src_path.is_empty() {
        eprintln!("error: please input the file name.");
        return;
    }

    // 检查文件是否存在
    let file_path = Path::new(src_path);
    if !file_path.exists() {
        eprintln!("Error: The file: {} is not exist", file_path.display());
        return;
    }

    // 根据类型参数处理逻辑
    if type_arg == "-t" {
        if args.len() < 4 {
            eprintln!("Error: Please input package name while analyse trace only.");
            return;
        }
        let package_name = &args[3];
        if package_name.is_empty() {
            eprintln!("Error: Please input package name while analyse trace only.");
            return;
        }
        // analyse_trace(file_path, package_name);
    } else {
        parse_log(file_path, &args);
    }

    println!("Done!");
}

fn print_help() {
    println!("Usage: program_name [-h] [-t] <file_path> [package_name]");
    println!("Options:");
    println!("  -h       Print this help message");
    println!("  -t       Analyse trace with package name");
}

// 定义 ANRResultBean 结构体
struct ANRResultBean {
    process_name: String,
    traces: Vec<Vec<String>>,
    log_file_paths: Vec<String>,
}

impl ANRResultBean {
    fn new(process_name: String) -> Self {
        ANRResultBean {
            process_name,
            traces: Vec::new(),
            log_file_paths: Vec::new(),
        }
    }

    fn add_traces(&mut self, traces: Vec<String>) -> usize {
        self.traces.push(traces);
        self.traces.len() - 1
    }

    fn add_log_file_path(&mut self, path: String, index: usize) {
        if index < self.log_file_paths.len() {
            self.log_file_paths[index] = path;
        } else {
            self.log_file_paths.push(path);
        }
    }

    fn compare_trace(&self, traces: &Vec<String>) -> isize {
        // 实现比较逻辑
        -1 // 占位返回值
    }

    fn write_to_file(&self, writer: &mut BufWriter<File>) -> io::Result<()> {
        writeln!(writer, "Process: {}", self.process_name)?;
        for trace in &self.traces {
            writeln!(writer, "Trace: {:?}", trace)?;
        }
        Ok(())
    }
}

// 解析日志文件
fn parse_log(path: &Path, args: &[String]) {
    let mut anr_result_bean_list = Vec::new();

    if !path.is_dir() {
        if let Some(result) = parse_single_log(path, args) {
            anr_result_bean_list.push(result);
        }
        // return anr_result_bean_list;
    }

    let mut item_list = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_path = entry.path();
                if !file_path.to_string_lossy().contains("summary.txt") {
                    if let Some(mut temp) = parse_single_log(&file_path, args) {
                        item_list.append(&mut temp);
                    }
                }
            }
        }
    }

    if !item_list.is_empty() {
        let list = turn_result_item_to_anr_list(item_list);
        // anr_result_bean_list.extend(list);
    }

    // write_summary(&anr_result_bean_list, path);
    // anr_result_bean_list
}

// 解析单个日志文件
fn parse_single_log(path: &Path, args: &[String]) -> Option<Vec<ResultItemBean>> {
    let path_str = path.to_string_lossy().to_string();
    let mut new_path = path_str.clone();

    if path_str.ends_with(".zip") {
        new_path = path_str[..path_str.len() - 4].to_string();
        // 调用解压逻辑（需要实现）
    } else if path_str.ends_with(".tar.gz") {
        new_path = path_str[..path_str.len() - 7].to_string();
        // 调用解压逻辑（需要实现）
    }

    let scr_file = Path::new(&new_path);
    // 调用日志分析逻辑（需要实现）
    Some(vec![ResultItemBean::with_details(
        "example_process".to_string(),
        vec!["trace1".to_string(), "trace2".to_string()],
        new_path,
    )])
}

// 写入总结文件
fn write_summary(anr_result_bean_list: &[ANRResultBean], path: &Path) {
    if anr_result_bean_list.is_empty() {
        return;
    }

    let summary_path = path.join("summary.txt");
    if let Ok(file) = File::create(&summary_path) {
        let mut writer = BufWriter::new(file);
        for anr_bean in anr_result_bean_list {
            if let Err(e) = anr_bean.write_to_file(&mut writer) {
                eprintln!("Failed to write to file: {}", e);
            }
        }
        println!("Summary written to: {}", summary_path.display());
    }
}

// 转换 ResultItemBean 列表为 ANRResultBean 列表
fn turn_result_item_to_anr_list(item_list: Vec<ResultItemBean>) -> Vec<ANRResultBean> {
    let mut anr_list = Vec::new();
    let mut process_to_index = HashMap::new();

    for item in item_list {
        let process_name = item.get_process_name().to_string();
        let trace_list = item.get_trace_list().clone();
        let out_path = item.get_out_path().to_string();

        if let Some(&index) = process_to_index.get(&process_name) {
            let anr_bean: &mut ANRResultBean = &mut anr_list[index];
            let trace_index = anr_bean.compare_trace(&trace_list);
            if trace_index < 0 {
                anr_bean.add_traces(trace_list);
            }
            anr_bean.add_log_file_path(out_path, trace_index as usize);
        } else {
            let mut anr_bean = ANRResultBean::new(process_name.clone());
            let index = anr_bean.add_traces(trace_list);
            anr_bean.add_log_file_path(out_path, index);
            process_to_index.insert(process_name, anr_list.len());
            anr_list.push(anr_bean);
        }
    }

    anr_list
}

// fn main() {
//     let path = Path::new("path/to/logs");
//     let args = vec!["arg1".to_string(), "arg2".to_string()];
//     let anr_results = parse_log(path, &args);
//     println!("Parsed {} ANR results", anr_results.len());
// }
