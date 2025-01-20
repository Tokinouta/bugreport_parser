#[derive(Debug, Default)]
pub struct ResultItemBean {
    process_name: String,
    trace_list: Vec<String>,
    out_path: String,
}

impl ResultItemBean {
    // 构造函数
    pub fn new() -> Self {
        ResultItemBean {
            process_name: String::new(),
            trace_list: Vec::new(),
            out_path: String::new(),
        }
    }

    pub fn with_details(process_name: String, trace_list: Vec<String>, out_path: String) -> Self {
        ResultItemBean {
            process_name,
            trace_list,
            out_path,
        }
    }

    // 设置 process_name
    pub fn set_process_name(&mut self, name: String) {
        self.process_name = name;
    }

    // 添加 trace 到 trace_list
    pub fn set_trace_list(&mut self, line: String) {
        self.trace_list.push(line);
    }

    // 设置 out_path
    pub fn set_out_path(&mut self, path: String) {
        self.out_path = path;
    }

    // 获取 process_name
    pub fn get_process_name(&self) -> &str {
        &self.process_name
    }

    // 获取 trace_list
    pub fn get_trace_list(&self) -> &Vec<String> {
        &self.trace_list
    }

    // 获取 out_path
    pub fn get_out_path(&self) -> &str {
        &self.out_path
    }

    // 获取 process_name
    pub fn get_process_name_mut(&mut self) -> &mut str {
        &mut self.process_name
    }

    // 获取 trace_list
    pub fn get_trace_list_mut(&mut self) -> &mut Vec<String> {
        &mut self.trace_list
    }

    // 获取 out_path
    pub fn get_out_path_mut(&mut self) -> &mut str {
        &mut self.out_path
    }
}
