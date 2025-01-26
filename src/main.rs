use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use clap::Parser;
use cli_parser::{Cli, Mode};
use models::anr_result_bean::ANRResultBean;
use models::result_item_bean::ResultItemBean;
use trace_analysis::TraceAnalysis;

pub mod models;
pub mod trace_analysis;
pub mod utils;
pub mod cli_parser;

fn main() {
    let args = Cli::parse();

    if args.repl {
        println!("Welcome to the Rust REPL!");
        repl();
        return;
    }

    // 检查文件路径是否为空
    if args.file_path.is_none() {
        eprintln!("Error: Please input the file name.");
        return;
    }

    // 检查文件是否存在
    let p = args.file_path.unwrap();
    let file_path = Path::new(&p);
    if !file_path.exists() {
        eprintln!("Error: The file '{}' does not exist.", file_path.display());
        return;
    }

    // 根据模式处理逻辑
    match args.mode {
        Mode::Parse => {
            if let Some(process_name) = args.process_name {
                parse_log(file_path, &[process_name]);
            } else {
                eprintln!("Error: Please provide a process name for parse mode.");
            }
        }
        Mode::AnalyseTrace => {
            if let Some(process_name) = args.process_name {
                let mut analysis = TraceAnalysis::new();
                // analysis.analyse_trace(file_path, &process_name);
            } else {
                eprintln!("Error: Please provide a process name for analyse trace mode.");
            }
        }
    }

    println!("Done!");
}

fn print_help() {
    println!("-h : Help");
    println!("<file_path> : Parse log file.");
    println!("<file_path> <process_name> -anr : Parse process anr.");
    println!("<file_path> <process_name> -kill : Parse process kill.");
    println!("<file_path> <process_name> -kill -s: Parse process kill and out to terminal.");
    println!("-t <trace_file_path> <process_name> : Parse traces file.");
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
        let mut trace_list = item.get_trace_list().clone();
        let out_path = item.get_out_path().to_string();

        if let Some(&index) = process_to_index.get(&process_name) {
            let anr_bean: &mut ANRResultBean = &mut anr_list[index];
            let trace_index = anr_bean.compare_trace(&mut trace_list);
            if trace_index.is_none() {
                anr_bean.add_traces(&trace_list);
            }
            anr_bean.add_log_file_path(out_path, trace_index.unwrap());
        } else {
            let mut anr_bean = ANRResultBean::new();
            anr_bean.set_process_name(process_name.clone());
            let index = anr_bean.add_traces(&mut trace_list);
            anr_bean.add_log_file_path(out_path, index);
            process_to_index.insert(process_name, anr_list.len());
            anr_list.push(anr_bean);
        }
    }

    anr_list
}

fn repl() {
    loop {
        // 提示用户输入
        print!("> ");
        io::stdout().flush().unwrap();

        // 读取用户输入
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                // 去除输入中的换行符
                let input = input.trim();

                // 如果用户输入 "exit"，退出 REPL
                if input == "exit" {
                    println!("Goodbye!");
                    break;
                }

                // 处理输入并执行相应的操作
                let result = evaluate_input(input);

                // 输出结果
                println!("{}", result);
            }
            Err(error) => {
                eprintln!("Error reading input: {}", error);
            }
        }
    }
}

fn evaluate_input(input: &str) -> String {
    // 这里可以添加更复杂的逻辑来解析和执行输入
    // 目前只是简单地返回输入的内容
    format!("You entered: {}", input)
}
