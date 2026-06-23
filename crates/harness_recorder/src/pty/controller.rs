use anyhow::Result;
use std::time::Duration;

use crate::script::TerminalSettings;
use super::Terminal;

pub struct TerminalController {
    terminal: Terminal,
}

impl TerminalController {
    pub fn new(settings: &TerminalSettings) -> Result<Self> {
        let terminal = Terminal::new(settings)?;
        Ok(Self { terminal })
    }
    
    pub async fn execute_command(&mut self, command: &str) -> Result<()> {
        log::debug!("Executing command: {}", command);
        self.terminal.execute_command(command).await
    }
    
    pub async fn type_text(&mut self, text: &str, speed: Duration) -> Result<()> {
        log::debug!("Typing text: {} (speed: {:?})", text, speed);
        self.terminal.type_text(text, speed).await
    }
    
    pub fn get_output(&self) -> String {
        self.terminal.get_output()
    }
    
    pub fn get_size(&self) -> (u16, u16) {
        self.terminal.get_size()
    }
    
    pub async fn wait_for_output(&self, pattern: &str, timeout: Duration) -> Result<bool> {
        self.terminal.wait_for_output(pattern, timeout).await
    }
    
    pub fn clear_output_buffer(&self) {
        self.terminal.clear_buffer();
    }
    
    pub fn get_terminal_ref(&self) -> &Terminal {
        &self.terminal
    }
}