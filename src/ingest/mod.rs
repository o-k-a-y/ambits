pub mod claude;

use std::path::PathBuf;

use crate::tracking::ReadDepth;

/// A parsed agent tool call event.
#[derive(Debug, Clone)]
pub struct AgentToolCall {
    pub agent_id: String,
    pub tool_name: String,
    pub file_path: Option<PathBuf>,
    pub read_depth: ReadDepth,
    pub description: String,
    pub timestamp_str: String,
}

/// Trait for agent event sources.
/// Implement this to support different agent frameworks.
pub trait AgentEventSource {
    /// Parse all events from existing log files.
    fn parse_existing(&self) -> color_eyre::Result<Vec<AgentToolCall>>;
}
