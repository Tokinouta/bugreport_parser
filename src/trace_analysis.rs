use std::collections::HashMap;

use crate::models::log_item_bean::LogItemBean;

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
