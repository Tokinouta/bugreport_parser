use std::{collections::HashMap, fs::File, io::{BufRead, BufReader, BufWriter}, path::Path};

use crate::models::{log_item_bean::LogItemBean, result_item_bean::ResultItemBean};

const BINDER_TRANSACT: &str = "$Stub$Proxy.";
const BINDER_PROXY: &str = "$Proxy.";
const TRANSACT: &str = ".onTransact";
const BIND_EXEC_TRANSACT: &str = "android.os.Binder.execTransact";
const HELD_BY_TID: &str = "held by tid=";
const HELD_BY_THREAD: &str = "held by thread ";
const PRE_TID: &str = "tid=";
const WAITING_TO_LOCK: &str = "waiting to lock <";
const LOCKED: &str = "- locked <";
const PREFIX_PID: &str = "----- pid ";
const PREFIX_CMD: &str = "Cmd line: ";

#[derive(Debug, Default)]
struct TraceAnalysis {
    current_pid_line: String,
    current_cmd_line: String,
    last_pid_line: String,
    last_cmd_line: String,
    current_log_bean: Option<LogItemBean>,
    lock_map: HashMap<String, bool>,
}

impl TraceAnalysis {
    // 构造函数
    fn new() -> Self {
        TraceAnalysis {
            ..Default::default()
        }
    }

    // 设置 current_pid_line
    fn set_current_pid_line(&mut self, pid_line: String) {
        self.current_pid_line = pid_line;
    }

    // 获取 current_pid_line
    fn get_current_pid_line(&self) -> &str {
        &self.current_pid_line
    }

    // 设置 current_cmd_line
    fn set_current_cmd_line(&mut self, cmd_line: String) {
        self.current_cmd_line = cmd_line;
    }

    // 获取 current_cmd_line
    fn get_current_cmd_line(&self) -> &str {
        &self.current_cmd_line
    }

    // 设置 last_pid_line
    fn set_last_pid_line(&mut self, pid_line: String) {
        self.last_pid_line = pid_line;
    }

    // 获取 last_pid_line
    fn get_last_pid_line(&self) -> &str {
        &self.last_pid_line
    }

    // 设置 last_cmd_line
    fn set_last_cmd_line(&mut self, cmd_line: String) {
        self.last_cmd_line = cmd_line;
    }

    // 获取 last_cmd_line
    fn get_last_cmd_line(&self) -> &str {
        &self.last_cmd_line
    }

    // 设置 current_log_bean
    fn set_current_log_bean(&mut self, log_bean: LogItemBean) {
        self.current_log_bean = Some(log_bean);
    }

    // 获取 current_log_bean
    fn get_current_log_bean(&self) -> Option<&LogItemBean> {
        self.current_log_bean.as_ref()
    }

    // 设置 lock_map 的值
    fn set_lock_map_value(&mut self, key: String, value: bool) {
        self.lock_map.insert(key, value);
    }

    // 获取 lock_map 的值
    fn get_lock_map_value(&self, key: &str) -> Option<bool> {
        self.lock_map.get(key).copied()
    }
}

impl TraceAnalysis {
    // 分析多个 LogItemBean
    fn analyse_trace_list(
        &mut self,
        src_file: &Path,
        bean_list: &mut [LogItemBean],
        result_list: &mut Vec<ResultItemBean>,
    ) -> Vec<i32> {
        let mut reasons = Vec::new();

        if bean_list.is_empty() {
            reasons.push(-2); // 如果 bean_list 为空，返回 -2
            return reasons;
        }

        for log_bean in bean_list {
            let mut item = ResultItemBean::new();
            item.set_process_name(log_bean.get_process_name().unwrap().to_string());
            item.set_out_path(src_file.parent().unwrap().to_string_lossy().to_string());

            let reason =
                self.analyse_trace(src_file, log_bean, src_file.parent().unwrap(), &mut item);
            reasons.push(reason);
            result_list.push(item);
        }

        reasons
    }

    // 分析单个 LogItemBean
    fn analyse_trace(
        &mut self,
        src_file: &Path,
        log_bean: &mut LogItemBean,
        out_folder: &Path,
        item: &mut ResultItemBean,
    ) -> i32 {
        if !src_file.exists() || !src_file.is_file() {
            return -1;
        }

        self.current_log_bean = Some(log_bean.clone());

        let out_dir = if out_folder.to_string_lossy().is_empty() {
            src_file.parent().unwrap().to_path_buf()
        } else {
            out_folder.to_path_buf()
        };

        let out_filename = if let Some(time) = log_bean.get_time() {
            format!("result_trace_{}_{}", log_bean.get_process_name().unwrap(), time)
        } else {
            format!("result_trace_{}", log_bean.get_process_name().unwrap())
        };

        let out_file = out_dir.join(out_filename);

        if let Ok(file) = File::create(&out_file) {
            let mut writer = BufWriter::new(file);
            let main_reason = self.analyse_trace_internal(src_file, log_bean, &mut writer, item);
            println!("Output file: {:?}", out_file);
            main_reason
        } else {
            -1
        }
    }

    // 内部分析逻辑
    fn analyse_trace_internal(
        &mut self,
        src_file: &Path,
        log_bean: &mut LogItemBean,
        writer: &mut BufWriter<File>,
        result_bean: &mut ResultItemBean,
    ) -> i32 {
        // this.mCurrentLogBean = logBean; Consider how to convert this to Rust 
        if let Ok(file) = File::open(src_file) {
            let reader = BufReader::new(file);
            let start_line = if let Some(pid) = log_bean.get_pid() {
                format!("----- pid {} at ", pid)
            } else {
                format!("Cmd line: {}", log_bean.get_process_name().unwrap())
            };

            for line in reader.lines().flatten() {
                if line.starts_with(&start_line) {
                    if line.starts_with("----- pid ") {
                        self.current_pid_line = line.clone();
                    } else if line.starts_with("Cmd line: ") {
                        self.current_cmd_line = line.clone();
                    }

                    if log_bean.get_time().is_none() {
                        if let Some(start) = self.current_pid_line.find("at") {
                            if let Some(end) = self.current_pid_line.rfind(" -----") {
                                let time = &self.current_pid_line[start + 3..end];
                                log_bean.set_time(time.to_string());
                            }
                        }
                    }
                }
            }
            0 // 成功
        } else {
            -1 // 失败
        }
    }
}
