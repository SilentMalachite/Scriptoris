use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub content: String,
    pub message_type: MessageType,
    pub created_at: Instant,
    pub auto_clear_duration: Option<Duration>,
}

impl StatusMessage {
    pub fn new(content: String, message_type: MessageType) -> Self {
        let auto_clear_duration = Self::default_duration_for_type(&message_type);
        Self {
            content,
            message_type,
            created_at: Instant::now(),
            auto_clear_duration,
        }
    }

    pub fn with_duration(content: String, message_type: MessageType, duration: Duration) -> Self {
        Self {
            content,
            message_type,
            created_at: Instant::now(),
            auto_clear_duration: Some(duration),
        }
    }

    pub fn permanent(content: String, message_type: MessageType) -> Self {
        Self {
            content,
            message_type,
            created_at: Instant::now(),
            auto_clear_duration: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(duration) = self.auto_clear_duration {
            self.created_at.elapsed() > duration
        } else {
            false
        }
    }

    fn default_duration_for_type(message_type: &MessageType) -> Option<Duration> {
        match message_type {
            MessageType::Info => Some(Duration::from_secs(3)),
            MessageType::Success => Some(Duration::from_secs(2)),
            MessageType::Warning => Some(Duration::from_secs(5)),
            MessageType::Error => Some(Duration::from_secs(7)),
        }
    }
}

#[derive(Clone)]
pub struct StatusManager {
    pub current_message: Option<StatusMessage>,
    pub mode_message: String,
}

impl StatusManager {
    pub fn new() -> Self {
        Self {
            current_message: None,
            mode_message: String::new(),
        }
    }

    pub fn set_info(&mut self, message: String) {
        self.current_message = Some(StatusMessage::new(message, MessageType::Info));
    }

    pub fn set_success(&mut self, message: String) {
        self.current_message = Some(StatusMessage::new(message, MessageType::Success));
    }

    pub fn set_warning(&mut self, message: String) {
        self.current_message = Some(StatusMessage::new(message, MessageType::Warning));
    }

    pub fn set_error(&mut self, message: String) {
        self.current_message = Some(StatusMessage::new(message, MessageType::Error));
    }

    pub fn set_permanent(&mut self, message: String, message_type: MessageType) {
        self.current_message = Some(StatusMessage::permanent(message, message_type));
    }

    pub fn set_mode_message(&mut self, message: String) {
        self.mode_message = message;
    }

    pub fn clear(&mut self) {
        self.current_message = None;
    }

    pub fn update(&mut self) {
        if let Some(ref message) = self.current_message {
            if message.is_expired() {
                self.current_message = None;
            }
        }
    }

    pub fn get_current_message(&self) -> Option<&StatusMessage> {
        self.current_message.as_ref()
    }

    pub fn get_mode_message(&self) -> &str {
        &self.mode_message
    }

    pub fn has_message(&self) -> bool {
        self.current_message.is_some()
    }
}

impl Default for StatusManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_status_manager_creation() {
        let manager = StatusManager::new();
        assert!(!manager.has_message());
        assert_eq!(manager.get_mode_message(), "");
    }

    #[test]
    fn test_message_types() {
        let mut manager = StatusManager::new();
        
        manager.set_info("Info message".to_string());
        let message = manager.get_current_message().unwrap();
        assert_eq!(message.message_type, MessageType::Info);
        assert_eq!(message.content, "Info message");

        manager.set_success("Success message".to_string());
        let message = manager.get_current_message().unwrap();
        assert_eq!(message.message_type, MessageType::Success);

        manager.set_warning("Warning message".to_string());
        let message = manager.get_current_message().unwrap();
        assert_eq!(message.message_type, MessageType::Warning);

        manager.set_error("Error message".to_string());
        let message = manager.get_current_message().unwrap();
        assert_eq!(message.message_type, MessageType::Error);
    }

    #[test]
    fn test_auto_clear() {
        let mut manager = StatusManager::new();
        manager.set_info("Test message".to_string());
        
        assert!(manager.has_message());
        
        // Message shouldn't be expired immediately
        let message = manager.get_current_message().unwrap();
        assert!(!message.is_expired());
    }

    #[test]
    fn test_permanent_message() {
        let mut manager = StatusManager::new();
        manager.set_permanent("Permanent message".to_string(), MessageType::Info);
        
        let message = manager.get_current_message().unwrap();
        assert!(!message.is_expired());
        assert!(message.auto_clear_duration.is_none());
    }

    #[test]
    fn test_mode_message() {
        let mut manager = StatusManager::new();
        manager.set_mode_message("-- INSERT --".to_string());
        assert_eq!(manager.get_mode_message(), "-- INSERT --");
    }

    #[test]
    fn test_clear() {
        let mut manager = StatusManager::new();
        manager.set_info("Test message".to_string());
        assert!(manager.has_message());
        
        manager.clear();
        assert!(!manager.has_message());
    }

    #[test]
    fn test_update_expired_message() {
        let mut manager = StatusManager::new();
        // Create a message with very short duration
        let short_duration = Duration::from_millis(1);
        let message = StatusMessage::with_duration("Test".to_string(), MessageType::Info, short_duration);
        manager.current_message = Some(message);
        
        // Wait for expiration
        thread::sleep(Duration::from_millis(10));
        
        manager.update();
        assert!(!manager.has_message());
    }
}