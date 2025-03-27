use bugreport::bugreport::Bugreport;
use clap::Parser;
use plugin::{timestamp_plugin, PluginRepo};
use std::{path::Path, sync::{Arc, Mutex}};

use cli_parser::{Cli, Mode};

pub mod bugreport;
pub mod cli_parser;
pub mod models;
pub mod plugin;
pub mod repl;
pub mod trace_analysis;
pub mod utils;

fn main() {
    let args = Cli::parse();

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

    if args.repl {
        println!("Welcome to the Rust REPL!");
        repl::repl(file_path);
        return;
    }

    // 根据模式处理逻辑
    match args.mode {
        Mode::Parse => {
            if let Some(process_name) = args.process_name {
                trace_analysis::parse_log(file_path, &[process_name]);
            } else {
                eprintln!("Error: Please provide a process name for parse mode.");
            }
        }
        Mode::AnalyseTrace => {
            if let Some(_) = args.process_name {
                // let mut analysis = TraceAnalysis::new();
                // analysis.analyse_trace(file_path, &process_name);
            } else {
                eprintln!("Error: Please provide a process name for analyse trace mode.");
            }
        }
        Mode::Bugreport => {
            let mut bugreport = Bugreport::new(file_path).unwrap();
            let _ = bugreport.load();
            let plugin = plugin::input_focus_plugin::InputFocusPlugin::new();
            let timestamp_plugin = timestamp_plugin::TimestampPlugin::new();
            PluginRepo::register(Arc::new(Mutex::new(plugin)));
            PluginRepo::register(Arc::new(Mutex::new(timestamp_plugin)));
            PluginRepo::analyze_all(&bugreport);
            println!("Plugin report:");
            println!("{}", PluginRepo::report_all());
        }
    }

    println!("Done!");
}

// fn print_help() {
//     println!("-h : Help");
//     println!("<file_path> : Parse log file.");
//     println!("<file_path> <process_name> -anr : Parse process anr.");
//     println!("<file_path> <process_name> -kill : Parse process kill.");
//     println!("<file_path> <process_name> -kill -s: Parse process kill and out to terminal.");
//     println!("-t <trace_file_path> <process_name> : Parse traces file.");
// }
