use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a point-in-time snapshot of an actor's metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorMetrics {
    /// Unique identifier for this actor
    pub id: String,
    /// Human-readable name/type of the actor
    pub actor_type: String,
    /// Current status of the actor
    pub status: ActorStatus,
    /// Timestamp when the actor was spawned (unix epoch milliseconds)
    pub spawned_at: u64,
    /// Timestamp when the actor was shut down (unix epoch milliseconds, None if still alive)
    pub shut_down_at: Option<u64>,
    /// Total number of messages successfully processed by this actor
    pub messages_processed: u64,
    /// Total number of messages sent by this actor
    pub messages_sent: u64,
    /// List of recent events (last N events)
    pub recent_events: Vec<ActorEvent>,
}

/// Current state of an actor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorStatus {
    /// Actor is running and processing messages
    Running,
    /// Actor has been instructed to stop
    Stopped,
    /// Actor encountered an error and is no longer processing
    Failed,
}

/// Represents a significant event in an actor's lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorEvent {
    /// Timestamp when the event occurred (unix epoch milliseconds)
    pub timestamp: u64,
    /// Type of event
    pub event_type: EventType,
    /// Optional additional context
    pub details: Option<String>,
}

/// Types of events that can be tracked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    /// Actor was spawned
    Spawned,
    /// Actor processed a message
    MessageProcessed,
    /// Actor sent a message
    MessageSent { recipient_id: String },
    /// Actor was instructed to stop
    Stopped,
    /// Actor encountered an error
    Failed { reason: String },
    /// Actor completed shutdown
    ShutDown,
}

/// Complete snapshot of the actor system's state and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionData {
    /// Snapshot timestamp (unix epoch milliseconds)
    pub timestamp: u64,
    /// Map of actor ID -> ActorMetrics for all known actors
    pub actors: HashMap<String, ActorMetrics>,
    /// Total number of messages passed through the system
    pub total_messages_processed: u64,
    /// Total number of messages sent
    pub total_messages_sent: u64,
}

impl ActorMetrics {
    /// Create a new actor metrics entry
    pub fn new(id: String, actor_type: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id,
            actor_type,
            status: ActorStatus::Running,
            spawned_at: now,
            shut_down_at: None,
            messages_processed: 0,
            messages_sent: 0,
            recent_events: vec![ActorEvent {
                timestamp: now,
                event_type: EventType::Spawned,
                details: None,
            }],
        }
    }

    /// Record that this actor processed a message
    pub fn record_message_processed(&mut self) {
        self.messages_processed += 1;
        self.add_event(EventType::MessageProcessed, None);
    }

    /// Record that this actor sent a message to another actor
    pub fn record_message_sent(&mut self, recipient_id: String) {
        self.messages_sent += 1;
        self.add_event(
            EventType::MessageSent {
                recipient_id: recipient_id.clone(),
            },
            None,
        );
    }

    /// Record a stop event
    pub fn record_stopped(&mut self) {
        self.status = ActorStatus::Stopped;
        self.add_event(EventType::Stopped, None);
    }

    /// Record a failure event
    pub fn record_failed(&mut self, reason: String) {
        self.status = ActorStatus::Failed;
        self.add_event(
            EventType::Failed {
                reason: reason.clone(),
            },
            Some(reason),
        );
    }

    /// Record a shutdown event
    pub fn record_shutdown(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.shut_down_at = Some(now);
        self.add_event(EventType::ShutDown, None);
    }

    /// Get uptime in milliseconds
    pub fn uptime_ms(&self) -> u64 {
        let end = self.shut_down_at.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        });
        end.saturating_sub(self.spawned_at)
    }

    /// Internal helper to add an event and maintain a rolling window
    fn add_event(&mut self, event_type: EventType, details: Option<String>) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        self.recent_events.push(ActorEvent {
            timestamp: now,
            event_type,
            details,
        });

        // Keep only the last 100 events to avoid unbounded growth
        if self.recent_events.len() > 100 {
            self.recent_events.remove(0);
        }
    }
}

impl SupervisionData {
    /// Create a new, empty supervision snapshot
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            timestamp: now,
            actors: HashMap::new(),
            total_messages_processed: 0,
            total_messages_sent: 0,
        }
    }

    /// Add or get actor metrics
    pub fn get_or_create_actor(&mut self, id: String, actor_type: String) -> &mut ActorMetrics {
        let id_clone = id.clone();
        self.actors
            .entry(id)
            .or_insert_with(|| ActorMetrics::new_with_type(id_clone, actor_type))
    }

    /// Record a message being processed
    pub fn record_message_processed(&mut self, actor_id: &str) {
        if let Some(actor) = self.actors.get_mut(actor_id) {
            actor.record_message_processed();
        }
        self.total_messages_processed += 1;
    }

    /// Record a message being sent
    pub fn record_message_sent(&mut self, from_id: &str, to_id: &str) {
        if let Some(actor) = self.actors.get_mut(from_id) {
            actor.record_message_sent(to_id.to_string());
        }
        self.total_messages_sent += 1;
    }

    /// Get all currently running actors
    pub fn running_actors(&self) -> Vec<&ActorMetrics> {
        self.actors
            .values()
            .filter(|a| a.status == ActorStatus::Running)
            .collect()
    }

    /// Get summary statistics
    pub fn summary(&self) -> SupervisionSummary {
        let actors = self.actors.values().collect::<Vec<_>>();
        let active_count = actors
            .iter()
            .filter(|a| a.status == ActorStatus::Running)
            .count();
        let failed_count = actors
            .iter()
            .filter(|a| a.status == ActorStatus::Failed)
            .count();

        SupervisionSummary {
            total_actors: actors.len(),
            active_actors: active_count,
            stopped_actors: actors.len() - active_count - failed_count,
            failed_actors: failed_count,
            total_messages_processed: self.total_messages_processed,
            total_messages_sent: self.total_messages_sent,
        }
    }
}

impl Default for SupervisionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary statistics for the actor system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionSummary {
    /// Total number of actors tracked
    pub total_actors: usize,
    /// Number of currently active (Running) actors
    pub active_actors: usize,
    /// Number of stopped actors
    pub stopped_actors: usize,
    /// Number of failed actors
    pub failed_actors: usize,
    /// Total messages processed
    pub total_messages_processed: u64,
    /// Total messages sent
    pub total_messages_sent: u64,
}

// Helper trait method to avoid breaking existing code
impl ActorMetrics {
    /// Create a new actor metrics entry with a specific type
    fn new_with_type(id: String, actor_type: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id,
            actor_type,
            status: ActorStatus::Running,
            spawned_at: now,
            shut_down_at: None,
            messages_processed: 0,
            messages_sent: 0,
            recent_events: vec![ActorEvent {
                timestamp: now,
                event_type: EventType::Spawned,
                details: None,
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_metrics_creation() {
        let metrics = ActorMetrics::new("actor-1".to_string(), "TestActor".to_string());
        assert_eq!(metrics.id, "actor-1");
        assert_eq!(metrics.actor_type, "TestActor");
        assert_eq!(metrics.status, ActorStatus::Running);
        assert_eq!(metrics.messages_processed, 0);
        assert_eq!(metrics.recent_events.len(), 1);
    }

    #[test]
    fn test_record_message_processed() {
        let mut metrics = ActorMetrics::new("actor-1".to_string(), "TestActor".to_string());
        metrics.record_message_processed();
        assert_eq!(metrics.messages_processed, 1);
        assert_eq!(metrics.recent_events.len(), 2); // Spawned + MessageProcessed
    }

    #[test]
    fn test_record_message_sent() {
        let mut metrics = ActorMetrics::new("actor-1".to_string(), "TestActor".to_string());
        metrics.record_message_sent("actor-2".to_string());
        assert_eq!(metrics.messages_sent, 1);
        assert_eq!(metrics.recent_events.len(), 2); // Spawned + MessageSent
    }

    #[test]
    fn test_record_failed() {
        let mut metrics = ActorMetrics::new("actor-1".to_string(), "TestActor".to_string());
        metrics.record_failed("test error".to_string());
        assert_eq!(metrics.status, ActorStatus::Failed);
        assert_eq!(metrics.recent_events.len(), 2); // Spawned + Failed
    }

    #[test]
    fn test_supervision_data_summary() {
        let mut supervision = SupervisionData::new();
        let metrics1 = ActorMetrics::new("actor-1".to_string(), "TestActor".to_string());
        let mut metrics2 = ActorMetrics::new("actor-2".to_string(), "TestActor".to_string());
        metrics2.record_stopped();
        let mut metrics3 = ActorMetrics::new("actor-3".to_string(), "TestActor".to_string());
        metrics3.record_failed("error".to_string());

        supervision.actors.insert("actor-1".to_string(), metrics1);
        supervision.actors.insert("actor-2".to_string(), metrics2);
        supervision.actors.insert("actor-3".to_string(), metrics3);

        let summary = supervision.summary();
        assert_eq!(summary.total_actors, 3);
        assert_eq!(summary.active_actors, 1);
        assert_eq!(summary.stopped_actors, 1);
        assert_eq!(summary.failed_actors, 1);
    }

    #[test]
    fn test_event_rolling_window() {
        let mut metrics = ActorMetrics::new("actor-1".to_string(), "TestActor".to_string());
        // Add 110 message processed events
        for _ in 0..110 {
            metrics.record_message_processed();
        }
        // Should keep only last 100 + initial spawned event = 101
        assert!(metrics.recent_events.len() <= 101);
    }
}
