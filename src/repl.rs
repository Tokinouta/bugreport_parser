use std::path::Path;
use std::rc::Rc;
use rustyline::{DefaultEditor, error::ReadlineError};

use crate::bugreport::bugreport_txt::BugreportTxt;
use crate::bugreport::logcat::LogcatLine;


enum ReplStatus {
    Ready,
    Logcat,
}

struct ReplState {
    bugreport: BugreportTxt,
    status: ReplStatus,
    last_command: String,
    last_result: Rc<Vec<LogcatLine>>,
}

pub fn repl(path: &Path) {
    let mut rl = DefaultEditor::new().unwrap();
    let mut bugreport = match BugreportTxt::new(path) {
        Ok(bugreport) => bugreport,
        Err(_) => return,
    };

    match bugreport.load() {
        Ok(matches) => matches,
        Err(_) => return,
    };
    let mut state = ReplState {
        bugreport,
        status: ReplStatus::Ready,
        last_command: String::new(),
        last_result: Rc::new(Vec::new()),
    };

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                // 去除输入中的换行符
                let input = line.trim();

                // 如果用户输入 "exit"，退出 REPL
                if input == "exit" {
                    println!("Goodbye!");
                    break;
                }

                // 处理输入并执行相应的操作
                let result = evaluate_input(input, &mut state);

                // 输出结果
                println!("{}", result);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn evaluate_input(input: &str, state: &mut ReplState) -> String {
    // 这里可以添加更复杂的逻辑来解析和执行输入
    // 目前只是简单地返回输入的内容
    format!("You entered: {}", input);

    // get all the sections with name "EVENT LOG" or "SYSTEM LOG"
    let sections = state
        .bugreport
        .get_sections()
        .iter()
        .filter(|s| s.name == "EVENT LOG" || s.name == "SYSTEM LOG")
        .collect::<Vec<_>>();

    if input.starts_with("tag") {
        // 解析 tag 命令
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() != 2 {
            return "Invalid tag command. Usage: tag <tag_name>".to_string();
        }
        let tag_name = parts[1];

        // 执行 tag 命令
        // TODO: 需要优化，如果上一次查询的结果不为空，则直接使用上一次查询的结果
        // TODO: 需要优化，加入自动补全 tag，避免想不起来
        let results = if state.last_result.len() > 0 {
            state.bugreport.search_by_tag(tag_name)
        } else {
            state.bugreport.search_by_tag(tag_name)
        };
        state.last_result = Rc::new(results.unwrap());

        return format!("Tagged results: {:?}", state.last_result);
    }

    "Ok".to_string()
}
