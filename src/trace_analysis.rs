use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Read, Seek, Write},
    path::Path,
};

use crate::{
    models::{
        anr_result_bean::ANRResultBean,
        lock_bean::{HeldThread, LockBean},
        log_item_bean::LogItemBean,
        result_item_bean::ResultItemBean,
    },
    utils::file_utils,
};

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
pub struct TraceAnalysis {
    current_pid_line: String,
    current_cmd_line: String,
    last_pid_line: String,
    last_cmd_line: String,
    current_log_bean: Option<LogItemBean>,
    lock_map: HashMap<String, bool>,
}

impl TraceAnalysis {
    // 构造函数
    pub fn new() -> Self {
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
    pub fn analyse_trace_list(
        &mut self,
        src_file: &Path,
        bean_list: &mut [LogItemBean],
        result_list: &mut Vec<ResultItemBean>,
        out_folder: Option<&Path>,
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

            let reason = self.analyse_trace(
                src_file,
                log_bean,
                &mut item,
                out_folder.unwrap_or(src_file.parent().unwrap()),
            );
            reasons.push(reason);
            result_list.push(item);
        }

        reasons
    }

    // 分析单个 LogItemBean
    pub fn analyse_trace(
        &mut self,
        src_file: &Path,
        log_bean: &mut LogItemBean,
        item: &mut ResultItemBean,
        out_folder: &Path,
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
            format!(
                "result_trace_{}_{}",
                log_bean.get_process_name().unwrap(),
                time
            )
        } else {
            format!("result_trace_{}", log_bean.get_process_name().unwrap())
        };

        let out_file = out_dir.join(out_filename);

        if let Ok(file) = File::create(&out_file) {
            let mut writer = BufWriter::new(file);
            let main_reason = self.analyse_trace_internal(src_file, log_bean, item, &mut writer);
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
        result_bean: &mut ResultItemBean,
        writer: &mut BufWriter<File>,
    ) -> i32 {
        // this.mCurrentLogBean = logBean; Consider how to convert this to Rust
        if let Ok(file) = File::open(src_file) {
            let mut reader = BufReader::new(file);
            let start_line = if let Some(pid) = log_bean.get_pid() {
                format!("----- pid {} at ", pid)
            } else {
                format!("Cmd line: {}", log_bean.get_process_name().unwrap())
            };

            let mut previous_line = String::new();
            for line in reader.by_ref().lines().flatten() {
                if line.starts_with(&start_line) {
                    if line.starts_with("----- pid ") {
                        self.current_pid_line = line.clone();
                    } else if line.starts_with("Cmd line: ") {
                        self.current_pid_line = previous_line.clone();
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

                    if log_bean.get_time().is_some() && log_bean.get_pid().is_some() {
                        let end = line.rfind(" -----").unwrap();
                        let start = start_line.len();
                        let time = &line[start..end];
                        if !log_bean.time_in_frame(time, 30000) {
                            continue;
                        }
                    }
                }
                previous_line = line;
            }
            self.get_main(&mut reader, writer, src_file, result_bean) // 成功
        } else {
            -1 // 失败
        }
    }

    fn get_main(
        &mut self,
        reader: &mut BufReader<File>,
        writer: &mut BufWriter<File>,
        src_file: &Path,
        result_bean: &mut ResultItemBean,
    ) -> i32 {
        let mut is_main_mode = false;
        let mut type_code = 0;

        if self.lock_map.is_empty() {
            self.lock_map.clear();
        }

        let mut line = String::new();
        while reader.read_line(&mut line).unwrap() > 0 {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with("Cmd line: ") {
                self.current_cmd_line = line.to_string();
            }

            if line[1..].starts_with("main") {
                is_main_mode = true;
                break;
            }
        }

        if !is_main_mode {
            return -1;
        }

        self.write_process_info(writer).unwrap();
        file_utils::write_line_to_file(&line, writer).unwrap();

        let mut api = String::new();
        let mut package_name = String::new();
        let mut lock_bean = LockBean::new();
        result_bean.get_trace_list_mut().clear();
        let process_name = result_bean.get_process_name().to_string();
        let mut is_add_trace_line_continue = true;

        while reader.read_line(&mut line).unwrap() > 0 && !line.starts_with("\"") {
            file_utils::write_line_to_file(&line, writer).unwrap();

            if !process_name.is_empty() {
                if line.trim().starts_with("at") && is_add_trace_line_continue {
                    result_bean.get_trace_list_mut().push(line.to_string());
                }

                if line.contains(&process_name) {
                    is_add_trace_line_continue = false;
                }
            }

            if line.contains("$Stub$Proxy.") {
                type_code = 1;
                let start = line.find("$Stub$Proxy.").unwrap() + "$Stub$Proxy.".len();
                api = line[start..line.rfind("(").unwrap()].to_string();
                package_name = line[..line.find("$Proxy.").unwrap()].to_string();
                continue;
            }

            if line.contains("waiting to lock <") {
                type_code = 2;
                self.get_lock_from_line(&line, &mut lock_bean);
                continue;
            }

            if line.contains("- locked <") {
                self.get_lock_from_line(&line, &mut lock_bean);
                let lock_key =
                    line[line.find("<").unwrap() + 1..line.find(">").unwrap()].to_string();
                self.lock_map.insert(lock_key, true);
            }
        }

        match type_code {
            1 => {
                self.binder_call_timeout(&api, &package_name, writer, src_file);
            }
            2 => {
                let mut model_lines = Vec::new();
                if let Ok(file) = File::open(src_file) {
                    let mut reader = BufReader::new(file);
                    self.analyse_trace_by_lock(
                        &mut lock_bean,
                        &mut model_lines,
                        writer,
                        src_file,
                        &mut reader,
                    );
                }
            }
            _ => {}
        }

        type_code
    }

    // Binder 调用超时逻辑
    fn binder_call_timeout(
        &mut self,
        api: &str,
        remote_package: &str,
        writer: &mut BufWriter<File>,
        src_file: &Path,
    ) -> io::Result<()> {
        if api.is_empty() || remote_package.is_empty() {
            return Ok(());
        }

        let remote_package = format!("{}.onTransact", remote_package);

        let file = File::open(src_file)?;
        let mut reader = BufReader::new(file);
        let mut mode_lines = Vec::new();
        let mut has_api = false;
        let mut has_package = false;
        let mut has_binder_transact = false;

        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            if line.starts_with("----- pid ") {
                self.current_pid_line = line.trim().to_string();
            } else if line.starts_with("Cmd line: ") {
                self.current_cmd_line = line.trim().to_string();
            }

            if line.starts_with("\"") {
                mode_lines.clear();
                mode_lines.push(self.current_pid_line.clone());
                mode_lines.push(self.current_cmd_line.clone());
                has_api = false;
                has_package = false;
                has_binder_transact = false;
            }

            mode_lines.push(line.trim().to_string());

            if line.contains(api) {
                has_api = true;
            }
            if line.contains(&remote_package) {
                has_package = true;
            }
            if line.contains("android.os.Binder.execTransact") {
                has_binder_transact = true;
            }

            if has_api && has_package && has_binder_transact {
                break;
            }
        }

        let mut lock_bean = LockBean::new();
        self.get_lock_from_model(&mut lock_bean, &mode_lines, writer);
        self.analyse_trace_by_lock(
            &mut lock_bean,
            &mut mode_lines,
            writer,
            src_file,
            &mut reader,
        );

        Ok(())
    }

    // 从行中提取锁信息
    fn get_lock_from_line(&self, line: &str, lock_bean: &mut LockBean) {
        if line.contains("waiting to lock <") {
            let object = line
                .split("waiting to lock <")
                .nth(1)
                .and_then(|s| s.split('>').next())
                .map(|s| s.trim().to_string());
            if let Some(obj) = object {
                lock_bean.add_waiting(obj);
            }
        } else if line.contains("- locked <") {
            let object = line
                .split("- locked <")
                .nth(1)
                .and_then(|s| s.split('>').next())
                .map(|s| s.trim().to_string());
            if let Some(obj) = object {
                lock_bean.add_lock(obj);
            }
        }

        let thread_tag = if line.contains("held by tid=") {
            Some("held by tid=")
        } else if line.contains("held by thread ") {
            Some("held by thread ")
        } else {
            None
        };

        if let Some(tag) = thread_tag {
            let substring = line.split(tag).nth(1).unwrap_or("");
            let tid = if let Some(end_index) = substring.find(' ') {
                format!("tid={}", &substring[..end_index])
            } else {
                format!("tid={}", substring)
            };

            let thread_name = if substring.contains('(') && substring.contains(')') {
                Some(
                    substring
                        .split('(')
                        .nth(1)
                        .and_then(|s| s.split(')').next())
                        .unwrap_or("")
                        .to_string(),
                )
            } else {
                None
            };

            lock_bean.add_waiting_thread(tid, thread_name);
        }
    }

    // 从模型行中提取锁信息
    fn get_lock_from_model(
        &self,
        lock_bean: &mut LockBean,
        model_lines: &[String],
        writer: &mut BufWriter<File>,
    ) -> io::Result<()> {
        if model_lines.is_empty() {
            println!("getLockFromModel modelLines is empty");
            return Ok(());
        }

        if writer.get_ref().metadata().is_err() || lock_bean.get_locked_objects().is_empty() {
            println!("getLockFromModel lockBean or writer is invalid");
            return Ok(());
        }

        lock_bean.clear();

        for line in model_lines {
            file_utils::write_line_to_file(line, writer)?;
            self.get_lock_from_line(line, lock_bean);
        }

        Ok(())
    }

    // 检查是否包含特殊锁
    fn contain_special_lock(&self, line: &str, search_lock: &[String]) -> bool {
        for lock in search_lock {
            if line.contains(lock) {
                return true;
            }
        }
        false
    }

    // 写入进程信息
    fn write_process_info(&self, writer: &mut BufWriter<File>) -> io::Result<()> {
        if !file_utils::is_empty(&self.current_pid_line)
            && self.current_pid_line != self.last_pid_line
        {
            file_utils::write_line_to_file(&self.current_pid_line, writer)?;
        }
        if !file_utils::is_empty(&self.current_cmd_line)
            && self.current_cmd_line != self.last_cmd_line
        {
            file_utils::write_line_to_file(&self.current_cmd_line, writer)?;
        }
        Ok(())
    }

    // 检查线程模型
    fn check_thread_model(&self, line: &str, held_threads: &[HeldThread]) -> bool {
        if file_utils::is_empty(line) {
            return false;
        }
        if held_threads.is_empty() {
            return true;
        }

        let mut is_in_model = false;
        for held_thread in held_threads {
            if !line.contains(&held_thread.tid) {
                continue;
            }
            match &held_thread.thread_name {
                Some(name) => {
                    if line.contains(name) {
                        is_in_model = true;
                        break;
                    }
                }
                None => {
                    is_in_model = true;
                    break;
                }
            }
        }

        if is_in_model {
            if let Some(start) = self.current_pid_line.find("at") {
                if let Some(end) = self.current_pid_line.find(" -----") {
                    let start = start + 3; // "at" 的长度是 2，加上空格
                    if start < end {
                        let time = &self.current_pid_line[start..end];
                        if let Some(log_bean) = &self.current_log_bean {
                            is_in_model = log_bean.time_in_frame(time, 21000);
                            return is_in_model;
                        }
                    }
                }
            }
        }
        false
    }

    // 分析基于锁的跟踪
    pub(crate) fn analyse_trace_by_lock(
        &mut self,
        lock_object: &mut LockBean,
        model_lines: &mut Vec<String>,
        writer: &mut BufWriter<File>,
        src_file: &Path,
        reader: &mut BufReader<File>,
    ) -> io::Result<()> {
        if lock_object.get_waiting_objects().is_empty() || !src_file.exists() {
            return Ok(());
        }

        model_lines.clear();

        // 将锁定的对象添加到锁映射中
        for object in lock_object.get_locked_objects() {
            self.lock_map.insert(object.clone(), true);
        }

        // 构建搜索锁列表
        let search_lock: Vec<String> = lock_object
            .get_waiting_objects()
            .iter()
            .map(|obj| format!("- locked <{}>", obj))
            .collect();

        let mut line = String::new();
        let mut is_find = false;
        let mut is_in_the_model = false;
        let temp_pid_line = self.current_pid_line.clone();
        let temp_cmd_line = self.current_cmd_line.clone();
        let mut pos_in_model_list = 0;

        // 读取文件内容
        while reader.read_line(&mut line)? > 0 {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with("----- pid ") {
                self.current_pid_line = line.to_string();
            } else if line.starts_with("Cmd line: ") {
                self.current_cmd_line = line.to_string();
            }

            if line.starts_with("\"") {
                if is_find {
                    break;
                }

                is_in_the_model = self.check_thread_model(line, lock_object.get_waiting_threads());
                if is_in_the_model {
                    pos_in_model_list = model_lines.len();
                    model_lines.push(self.current_pid_line.clone());
                    model_lines.push(self.current_cmd_line.clone());
                }
            }

            if is_in_the_model {
                model_lines.push(line.to_string());
                if self.contain_special_lock(line, &search_lock) {
                    is_find = true;
                }
            }
        }

        // 重置读取器
        reader.seek(io::SeekFrom::Start(0))?;

        // 恢复原始状态
        self.current_pid_line = temp_pid_line;
        self.current_cmd_line = temp_cmd_line;

        // 处理模型行
        if !model_lines.is_empty() {
            if is_find {
                model_lines.truncate(pos_in_model_list);
            }

            // 调用 get_lock_from_model 方法
            self.get_lock_from_model(lock_object, model_lines, writer)?;

            // 递归调用 analyse_trace_by_lock
            self.analyse_trace_by_lock(lock_object, model_lines, writer, src_file, reader)?;
        }

        Ok(())
    }
}

// 解析日志文件
pub fn parse_log(path: &Path, args: &[String]) {
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
