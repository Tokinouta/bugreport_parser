use chrono::{NaiveDateTime, Utc};
use std::fmt;

// 定义 LogItemBean 结构体
#[derive(Debug, Clone, Default)]
pub struct LogItemBean {
    time: Option<String>,
    pid: Option<String>,
    tid: Option<String>,
    description: Option<String>,
    content: Option<String>,
    process_name: Option<String>,
    reason: Option<String>,
}

impl LogItemBean {
    // 构造函数
    fn new() -> Self {
        LogItemBean {
            time: None,
            pid: None,
            tid: None,
            description: None,
            content: None,
            process_name: None,
            reason: None,
        }
    }

    // 带参数的构造函数
    fn with_details(
        time: String,
        pid: String,
        tid: String,
        description: String,
        content: String,
    ) -> Self {
        LogItemBean {
            time: Some(time),
            pid: Some(pid),
            tid: Some(tid),
            description: Some(description),
            content: Some(content),
            process_name: None,
            reason: None,
        }
    }

    // Getter 和 Setter 方法
    pub fn get_reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    pub fn set_reason(&mut self, reason: String) {
        self.reason = Some(reason);
    }

    pub fn get_process_name(&self) -> Option<&str> {
        self.process_name.as_deref()
    }

    pub fn set_process_name(&mut self, process_name: String) {
        self.process_name = Some(process_name);
    }

    pub fn get_time(&self) -> Option<&str> {
        self.time.as_deref()
    }

    pub fn set_time(&mut self, time: String) {
        self.time = Some(time);
    }

    pub fn get_pid(&self) -> Option<&str> {
        self.pid.as_deref()
    }

    pub fn set_pid(&mut self, pid: String) {
        self.pid = Some(pid);
    }

    pub fn get_tid(&self) -> Option<&str> {
        self.tid.as_deref()
    }

    pub fn set_tid(&mut self, tid: String) {
        self.tid = Some(tid);
    }

    pub fn get_description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    pub fn get_content(&self) -> Option<&str> {
        self.content.as_deref()
    }

    pub fn set_content(&mut self, content: String) {
        self.content = Some(content);
    }

    // 比较两个 LogItemBean 是否相等
    pub fn equals(&self, other: &LogItemBean, max_time_diff: i64) -> bool {
        if other.time.is_none() {
            return self.pid == other.pid;
        }
        if self.pid == other.pid && self.time_in_frame(other.get_time().unwrap(), max_time_diff) {
            return true;
        }
        false
    }

    // 检查时间是否在允许的范围内
    pub fn time_in_frame(&self, time2: &str, max_time_diff: i64) -> bool {
        let time1 = self.check_time(self.get_time().unwrap());
        let time2 = self.check_time(time2);

        if time1.is_none() || time2.is_none() {
            return false;
        }

        let format = "%Y-%m-%d %H:%M:%S";
        if let Ok(dt1) = NaiveDateTime::parse_from_str(&time1.unwrap(), format) {
            if let Ok(dt2) = NaiveDateTime::parse_from_str(&time2.unwrap(), format) {
                let diff = dt1.signed_duration_since(dt2).num_milliseconds().abs();
                return diff < max_time_diff;
            }
        }
        false
    }

    // 检查时间格式并补全年份
    fn check_time(&self, time: &str) -> Option<String> {
        if time.is_empty() {
            return None;
        }

        if time.find('-').unwrap() != 4 {
            let now = Utc::now();
            let year = now.format("%Y").to_string();
            return Some(format!("{}-{}", year, time));
        }
        Some(time.to_string())
    }
}

// 实现 Display trait 以便打印 LogItemBean
impl fmt::Display for LogItemBean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LogItemBean{{time='{}', pid='{}', description='{}', process_name='{}', reason='{}'}}",
            self.time.as_deref().unwrap_or(""),
            self.pid.as_deref().unwrap_or(""),
            self.description.as_deref().unwrap_or(""),
            self.process_name.as_deref().unwrap_or(""),
            self.reason.as_deref().unwrap_or("")
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::models::lock_bean::LockBean;
    use crate::trace_analysis::TraceAnalysis;
    use std::{
        fs::File,
        io::{BufReader, BufWriter},
        path::Path,
    };

    #[test]
    fn main() {
        // 示例文件路径
        let src_file = Path::new("path/to/src_file");

        // 创建 LockBean 实例
        let mut lock_object = LockBean::new();

        // 创建 BufWriter 实例
        if let Ok(file) = File::create("path/to/output_file") {
            let mut writer = BufWriter::new(file);

            // 创建 BufReader 实例
            if let Ok(file) = File::open(src_file) {
                let mut reader = BufReader::new(file);

                // 创建 TracesAnalyse 实例
                let mut analyser = TraceAnalysis::new();

                // 调用 analyse_trace_by_lock 方法
                if let Err(e) = analyser.analyse_trace_by_lock(
                    &mut lock_object,
                    &mut Vec::new(),
                    &mut writer,
                    src_file,
                    &mut reader,
                ) {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }
}
