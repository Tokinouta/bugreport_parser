use chrono::{DateTime, Local};

use crate::bugreport::bugreport::Bugreport;

use super::Plugin;

pub struct TimestampPlugin {
    timestamp: DateTime<Local>,
}

impl Plugin for TimestampPlugin {
    fn name(&self) -> &str {
        "TimestampPlugin"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn analyze(&mut self, bugreport: &Bugreport) {
        // Analyze the bug report and extract timestamps
        self.timestamp = bugreport.get_metadata().timestamp.clone();
        println!("Analyzed timestamps: {:?}", self.timestamp);
    }

    fn report(&self) -> String {
        format!("Bugreport timestamp: {}", self.timestamp.to_rfc3339())
    }
}

impl TimestampPlugin {
    pub fn new() -> Self {
        TimestampPlugin {
            timestamp: Local::now(),
        }
    }
}

mod test {
    use super::*;
    use crate::bugreport::bugreport::{test_setup_bugreport, Bugreport};
    use crate::bugreport::metadata::Metadata;

    #[test]
    fn test_timestamp_plugin() {
        let mut bugreport = test_setup_bugreport().unwrap();
        bugreport.load().unwrap();
        let mut plugin = TimestampPlugin::new();
        plugin.analyze(&bugreport);
        assert_eq!(plugin.report(), "Bugreport timestamp: 2024-08-16T10:02:11+08:00");
    }
}
