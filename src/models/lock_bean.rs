use std::fmt;

// 定义 HeldThread 结构体
#[derive(Debug, Clone)]
pub struct HeldThread {
    pub tid: String,
    pub thread_name: Option<String>,
}

impl HeldThread {
    // 构造函数
    fn new(tid: String, thread_name: Option<String>) -> Self {
        HeldThread { tid, thread_name }
    }
}

// 实现 Display trait 以便打印 HeldThread
impl fmt::Display for HeldThread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.thread_name {
            write!(f, "[ {} {} ]", self.tid, name)
        } else {
            write!(f, "[ {} ]", self.tid)
        }
    }
}

// 定义 LockBean 结构体
#[derive(Debug)]
pub struct LockBean {
    locked_objects: Vec<String>,
    waiting_objects: Vec<String>,
    waiting_threads: Vec<HeldThread>,
}

impl LockBean {
    // 构造函数
    pub fn new() -> Self {
        LockBean {
            locked_objects: Vec::new(),
            waiting_objects: Vec::new(),
            waiting_threads: Vec::new(),
        }
    }

    // 添加锁对象
    pub fn add_lock(&mut self, object: String) {
        self.locked_objects.push(object);
    }

    // 添加等待对象
    pub fn add_waiting(&mut self, object: String) {
        self.waiting_objects.push(object);
    }

    // 添加等待线程（带线程名）
    pub fn add_waiting_thread(&mut self, tid: String, thread_name: Option<String>) {
        self.waiting_threads.push(HeldThread::new(tid, thread_name));
    }

    // 获取锁对象列表
    pub fn get_locked_objects(&self) -> &Vec<String> {
        &self.locked_objects
    }

    // 获取等待对象列表
    pub fn get_waiting_objects(&self) -> &Vec<String> {
        &self.waiting_objects
    }

    // 获取等待线程列表
    pub fn get_waiting_threads(&self) -> &Vec<HeldThread> {
        &self.waiting_threads
    }

    // 清空所有数据
    pub fn clear(&mut self) {
        self.locked_objects.clear();
        self.waiting_objects.clear();
        self.waiting_threads.clear();
    }
}

// 实现 Display trait 以便打印 LockBean
impl fmt::Display for LockBean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[ {{{}}} {{{}}} {{{}}} ]",
            self.locked_objects.join(", "),
            self.waiting_objects.join(", "),
            self.waiting_threads
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
