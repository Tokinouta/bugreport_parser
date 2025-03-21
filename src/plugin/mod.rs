use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

use crate::bugreport::logcat::LogcatLine;

// Define the plugin trait
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn register(&self);
    fn analyze(&self, logcat: &[LogcatLine]);
    fn on_event(&self, event: &str);
}

// Global plugin repository
lazy_static! {
    static ref PLUGIN_REPO: Mutex<Vec<Arc<dyn Plugin>>> = Mutex::new(Vec::new());
}

// Plugin repository manager
pub struct PluginRepo;

impl PluginRepo {
    /// Register a new plugin
    pub fn register(plugin: Arc<dyn Plugin>) {
        PLUGIN_REPO.lock().unwrap().push(plugin);
    }

    /// Get all registered plugins
    pub fn get_all() -> Vec<Arc<dyn Plugin>> {
        PLUGIN_REPO.lock().unwrap().clone()
    }

    /// Find a plugin by name
    pub fn find_by_name(name: &str) -> Option<Arc<dyn Plugin>> {
        PLUGIN_REPO
            .lock()
            .unwrap()
            .iter()
            .find(|p| p.name() == name)
            .map(Arc::clone)
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

        fn on_event(&self, event: &str) {
            println!("{} says: Event '{}' occurred!", self.name(), event);
        }

        fn version(&self) -> &str {
            todo!()
        }

        fn register(&self) {
            todo!()
        }

        fn analyze(&self, logcat: &[LogcatLine]) {
            todo!()
        }
    }

    struct LoggingPlugin;

    impl Plugin for LoggingPlugin {
        fn name(&self) -> &str {
            "LoggingPlugin"
        }

        fn on_event(&self, event: &str) {
            println!("[LOG] {}: {}", self.name(), event);
        }

        fn version(&self) -> &str {
            todo!()
        }

        fn register(&self) {
            todo!()
        }

        fn analyze(&self, logcat: &[LogcatLine]) {
            todo!()
        }
    }

    #[test]
    fn test_plugin_repo() {
        // Register plugins at startup
        PluginRepo::register(Arc::new(GreetingPlugin));
        PluginRepo::register(Arc::new(LoggingPlugin));

        // Demonstrate using all plugins
        let plugins = PluginRepo::get_all();
        println!("Registered {} plugins:", plugins.len());

        for plugin in &plugins {
            plugin.on_event("system_start");
        }

        // Demonstrate finding a specific plugin
        if let Some(logger) = PluginRepo::find_by_name("LoggingPlugin") {
            logger.on_event("custom_log_entry");
        }
    }
}
