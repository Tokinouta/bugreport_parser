use clap::{Parser, ValueEnum};

/// 一个命令行程序，用于解析日志文件或分析跟踪文件
#[derive(Parser, Debug)]
#[command(name = "log_parser", version = "1.0", author = "Your Name")]
pub struct Cli {
    /// 文件路径
    pub file_path: Option<String>,

    /// 操作模式
    #[arg(value_enum, short, long, value_parser, default_value = "parse")]
    pub mode: Mode,

    /// 进程名称（仅在某些模式下需要）
    #[arg(short, long, value_parser)]
    pub process_name: Option<String>,

    /// 是否启用 REPL 模式
    #[arg(short, long, action, default_value = "false")]
    pub repl: bool,
}

/// 支持的操作模式
#[derive(ValueEnum, Clone, Debug)]
pub enum Mode {
    Parse,
    AnalyseTrace,
    Bugreport
}
