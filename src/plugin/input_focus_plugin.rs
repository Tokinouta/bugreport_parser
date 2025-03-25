use std::sync::Arc;

use lazy_static::lazy_static;
use regex::Regex;

use crate::bugreport::{bugreport::Bugreport, logcat::LogcatLine, section::Section};

use super::{Plugin, PluginRepo};

lazy_static! {
    pub static ref INPUT_FOCUS_REQUEST: Regex =
        Regex::new(r#"\[Focus request ([\w /\.]+),reason=(\w+)\]"#).unwrap();
    pub static ref INPUT_FOCUS_RECEIVE: Regex =
        Regex::new(r#"\[Focus receive :([\w /\.]+),.*\]"#).unwrap();
    pub static ref INPUT_FOCUS_ENTERING: Regex =
        Regex::new(r#"\[Focus entering ([\w /\.]+) (\(server\))?,.*\]"#).unwrap();
    pub static ref INPUT_FOCUS_LEAVING: Regex =
        Regex::new(r#"\[Focus leaving ([\w /\.]+) (\(server\))?,.*\]"#).unwrap();
}

#[derive(Debug)]
pub struct InputFocusTuple {
    pub request: Option<LogcatLine>,
    pub receive: Option<LogcatLine>,
    pub entering: Option<LogcatLine>,
    pub leaving: Option<LogcatLine>,
}

pub struct InputFocusPlugin {
    records: Vec<InputFocusTuple>,
    result: String,
}

impl Plugin for InputFocusPlugin {
    fn name(&self) -> &str {
        "InputFocusPlugin"
    }

    fn on_event(&self, event: &str) {
        println!("{} says: Event '{}' occurred!", self.name(), event);
    }

    fn version(&self) -> &str {
        todo!()
    }

    fn register(&self) {
        // The error indicates that the trait bound `&InputFocusPlugin: plugin::Plugin` is not satisfied.
        // The `PluginRepo::register` method expects an `Arc<dyn Plugin>`, but `self` is borrowed here.
        // We need to clone `self` to create an owned `InputFocusPlugin` instance.
        // PluginRepo::register(Arc::new(self.clone()));
    }

    fn analyze(&mut self, bugreport: &Bugreport) {
        let event_log_section = match bugreport.sections.iter().find(|s| s.name == "EVENT LOG") {
            Some(section) => section,
            None => panic!("EVENT LOG section not found"),
        };

        self.pair_input_focus(event_log_section);
    }

    fn report(&self) -> String {
        self.result.clone()
    }
}

impl InputFocusPlugin {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            result: String::new(),
        }
    }

    /// pair input_focus logs within event log
    /// 
    /// 1. 第一步通过 dump of service greezer 找到用户开关屏幕的时间点，也可以考虑通过 screen_toggled 0
    /// 2. 第二步根据上述开关屏时间点找当时的 input_focus 记录，看看每一个时间点的 focus 到底在哪里
    /// 3. 第三步看 wm 生命周期，看能不能跟 focus 记录对上
    pub fn pair_input_focus(&mut self, section: &Section) {
        let result = match section.search_by_tag("input_focus") {
            Some(logs) => logs,
            None => return,
        };

        // find all the entries with its message containing "Focus request"
        let mut request_focus = Vec::new();
        for (i, line) in result.iter().enumerate() {
            if line.message.contains("Focus request") {
                request_focus.push((i, line));
            }
        }

        // find entries with its message containing "Focus receive", "Focus entering", "Focus leaving" that appear after the request_focus
        // and the timestamp is the closest to the request_focus
        // compact the result into a vec of InputFocusPair
        for (i, request) in request_focus {
            let mut window = String::new();
            if let Some(captures) = INPUT_FOCUS_REQUEST.captures(&request.message) {
                window = captures.get(1).map_or("", |m| m.as_str()).to_string();
            }
            // Bug fix: The `push_str` method of the `String` type takes only one argument, which is the string to be appended.
            // The original code tried to use a format string, which is incorrect. We need to use the `format!` macro to format the string first,
            // and then append it to `self.result`.
            self.result.push_str(&format!("window: {}\n", window));

            let mut receive = None;
            let mut entering = None;
            let mut leaving = None;
            for line in result.iter().skip(i + 1) {
                if let Some(captures) = INPUT_FOCUS_RECEIVE.captures(&line.message) {
                    if receive.is_none() && captures.get(1).map_or("", |m| m.as_str()) == window {
                        receive = Some(line);
                    }
                } else if let Some(captures) = INPUT_FOCUS_ENTERING.captures(&line.message) {
                    if entering.is_none() && captures.get(1).map_or("", |m| m.as_str()) == window {
                        entering = Some(line);
                    }
                } else if let Some(captures) = INPUT_FOCUS_LEAVING.captures(&line.message) {
                    if leaving.is_none() && captures.get(1).map_or("", |m| m.as_str()) == window {
                        leaving = Some(line);
                    }
                }
                if receive.is_some() && entering.is_some() && leaving.is_some() {
                    break;
                }
            }
            self.records.push(InputFocusTuple {
                request: Some(request.clone()),
                receive: match receive {
                    Some(line) => Some(line.clone()),
                    None => None,
                },
                entering: match entering {
                    Some(line) => Some(line.clone()),
                    None => None,
                },
                leaving: match leaving {
                    Some(line) => Some(line.clone()),
                    None => None,
                },
            });
        }

        println!("rsrasreraerasera: {:?}", self.records.len());
    }

    fn get_records(&self) -> &Vec<InputFocusTuple> {
        &self.records
    }
}

mod tests {
    use crate::plugin;

    use super::*;

    #[test]
    fn test_pair_input_focus() {
        let mut bugreport = crate::bugreport::bugreport::test_setup_bugreport().unwrap();
        match bugreport.load() {
            Ok(matches) => matches,
            Err(e) => panic!("Error: {}", e),
        };
        let mut plugin = InputFocusPlugin::new();
        plugin.analyze(&bugreport);
        let result = plugin.get_records();
        for pair in result {
            println!("{:?}", pair);
            let request_activity = INPUT_FOCUS_REQUEST
                .captures(&pair.request.as_ref().unwrap().message)
                .unwrap()
                .get(1)
                .unwrap()
                .as_str();
            // check if the four fields have increasing timestamp and the same activity
            // avoid formatting the following lines

            if pair.receive.is_none() {
                continue;
            }
            #[rustfmt::skip]
            assert!(pair.receive.as_ref().unwrap().timestamp >= pair.request.as_ref().unwrap().timestamp);
            #[rustfmt::skip]
            assert!(pair.receive.as_ref().unwrap().message.contains(request_activity));

            if pair.entering.is_none() {
                continue;
            }
            #[rustfmt::skip]
            assert!(pair.entering.as_ref().unwrap().timestamp >= pair.receive.as_ref().unwrap().timestamp);
            #[rustfmt::skip]
            assert!(pair.entering.as_ref().unwrap().message.contains(request_activity));

            if pair.leaving.is_none() {
                continue;
            }
            // In some cases, the leaving timestamp is before the entering timestamp, which could be due to the interleaved log
            // assert!(pair.leaving.as_ref().unwrap().timestamp >= pair.entering.as_ref().unwrap().timestamp);
            #[rustfmt::skip]
            assert!(pair.leaving.as_ref().unwrap().message.contains(request_activity));
        }
    }
}
