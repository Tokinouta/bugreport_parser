use lazy_static::lazy_static;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::bugreport::{bugreport_txt::BugreportTxt, logcat::LogcatLine};

pub mod input_focus_plugin;
pub mod timestamp_plugin;

// Define the plugin trait
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn analyze(&mut self, bugreport: &BugreportTxt);
    fn report(&self) -> String;
}

// Global plugin repository
lazy_static! {
    static ref PLUGIN_REPO: Mutex<Vec<Arc<Mutex<dyn Plugin>>>> = Mutex::new(Vec::new());
}

// Plugin repository manager
pub struct PluginRepo;

impl PluginRepo {
    /// Register a new plugin
    pub fn register(plugin: Arc<Mutex<dyn Plugin>>) {
        PLUGIN_REPO.lock().unwrap().push(plugin);
    }

    /// Get all registered plugins
    pub fn get_all() -> Vec<Arc<Mutex<dyn Plugin>>> {
        PLUGIN_REPO.lock().unwrap().clone()
    }

    /// Find a plugin by name
    pub fn find_by_name(name: &str) -> Option<Arc<Mutex<dyn Plugin>>> {
        PLUGIN_REPO
            .lock()
            .unwrap()
            .iter()
            .find(|p| p.try_lock().unwrap().name() == name)
            .map(Arc::clone)
    }

    pub fn analyze_all(bugreport: &BugreportTxt) {
        for plugin in PluginRepo::get_all() {
            let mut plugin = plugin.lock().unwrap();
            plugin.analyze(bugreport);
        }
    }

    pub fn report_all() -> String {
        let mut reports = Vec::new();
        for plugin in PluginRepo::get_all() {
            let plugin = plugin.lock().unwrap();
            reports.push(plugin.report());
        }
        reports.join("\n")
    }
}

mod test {
    use super::*;
    // Example plugin implementations
    struct GreetingPlugin;

    impl Plugin for GreetingPlugin {
        fn name(&self) -> &str {
            "GreetingPlugin"
        }

        fn version(&self) -> &str {
            todo!()
        }

        fn analyze(&mut self, _: &BugreportTxt) {
            todo!()
        }

        fn report(&self) -> String {
            format!("{} says: Reporteded!", self.name())
        }
    }

    struct LoggingPlugin;

    impl Plugin for LoggingPlugin {
        fn name(&self) -> &str {
            "LoggingPlugin"
        }

        fn version(&self) -> &str {
            todo!()
        }

        fn analyze(&mut self, _: &BugreportTxt) {
            todo!()
        }

        fn report(&self) -> String {
            format!("{} says: Reporteded!", self.name())
        }
    }

    #[test]
    fn test_plugin_repo() {
        // Register plugins at startup
        PluginRepo::register(Arc::new(Mutex::new(GreetingPlugin)));
        PluginRepo::register(Arc::new(Mutex::new(LoggingPlugin)));

        // Demonstrate using all plugins
        let plugins = PluginRepo::get_all();
        println!("Registered {} plugins:", plugins.len());

        for plugin in &plugins {
            plugin.lock().unwrap().report();
        }

        // Demonstrate finding a specific plugin
        if let Some(logger) = PluginRepo::find_by_name("LoggingPlugin") {
            logger.lock().unwrap().report();
        }
    }
}
