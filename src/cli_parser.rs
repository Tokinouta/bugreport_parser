use clap::{ValueEnum, Parser};
use std::path::Path;

/// 一个命令行程序，用于解析日志文件或分析跟踪文件
#[derive(Parser, Debug)]
#[command(name = "log_parser", version = "1.0", author = "Your Name")]
struct Cli {
    /// 文件路径
    #[arg(short, long, value_parser)]
    file_path: String,

    /// 操作模式
    #[arg(value_enum, short, long, value_parser, default_value = "parse")]
    mode: Mode,

    /// 进程名称（仅在某些模式下需要）
    #[arg(short, long, value_parser)]
    process_name: Option<String>,

    /// 是否启用 REPL 模式
    #[arg(short, long, action)]
    repl: bool,

    /// 是否输出到终端
    #[arg(short, long, action)]
    output_to_terminal: bool,
}

/// 支持的操作模式
#[derive(ValueEnum, Clone, Debug)]
enum Mode {
    Parse,
    AnalyseTrace,
}