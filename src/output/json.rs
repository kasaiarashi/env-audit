use anyhow::Result;

use crate::types::ScanReport;
use super::OutputFormatter;

pub struct JsonOutput {
    pub pretty: bool,
}

impl JsonOutput {
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }
}

impl OutputFormatter for JsonOutput {
    fn format(&self, report: &ScanReport) -> Result<String> {
        let output = if self.pretty {
            serde_json::to_string_pretty(report)?
        } else {
            serde_json::to_string(report)?
        };
        Ok(output)
    }
}
