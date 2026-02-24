use crate::supervision::{SupervisionData, ActorStatus};
use serde_json::json;
use std::collections::HashMap;

/// Export actor system supervision data to JSON format
pub fn to_json(supervision: &SupervisionData) -> Result<String, Box<dyn std::error::Error>> {
    let json_value = json!({
        "timestamp": supervision.timestamp,
        "summary": {
            "total_actors": supervision.actors.len(),
            "total_messages_processed": supervision.total_messages_processed,
            "total_messages_sent": supervision.total_messages_sent,
        },
        "actors": supervision.actors.iter().map(|(id, metrics)| {
            (id.clone(), json!({
                "actor_type": metrics.actor_type,
                "status": match metrics.status {
                    ActorStatus::Running => "Running",
                    ActorStatus::Stopped => "Stopped",
                    ActorStatus::Failed => "Failed",
                },
                "spawned_at": metrics.spawned_at,
                "shut_down_at": metrics.shut_down_at,
                "messages_processed": metrics.messages_processed,
                "messages_sent": metrics.messages_sent,
                "uptime_ms": metrics.uptime_ms(),
                "recent_events": metrics.recent_events.iter().map(|e| {
                    json!({
                        "timestamp": e.timestamp,
                        "event_type": format!("{:?}", e.event_type),
                        "details": e.details,
                    })
                }).collect::<Vec<_>>(),
            }))
        }).collect::<HashMap<_, _>>(),
    });

    Ok(serde_json::to_string_pretty(&json_value)?)
}

/// Export actor system supervision data to Graphviz DOT format
pub fn to_dot(supervision: &SupervisionData) -> String {
    let mut dot = String::from("digraph ActorSystem {\n");
    dot.push_str("  rankdir=LR;\n");
    dot.push_str("  node [shape=box, style=filled, fillcolor=lightblue];\n\n");

    // Add actor nodes
    for (id, metrics) in &supervision.actors {
        let label = format!(
            "{} ({})\\nStatus: {:?}\\nMessages: {}",
            metrics.actor_type,
            id,
            metrics.status,
            metrics.messages_processed
        );
        let color = match metrics.status {
            ActorStatus::Running => "lightgreen",
            ActorStatus::Stopped => "lightyellow",
            ActorStatus::Failed => "lightcoral",
        };
        dot.push_str(&format!(
            "  \"{}\" [label=\"{}\", fillcolor=\"{}\"];\n",
            id, label, color
        ));
    }

    dot.push_str("\n  // Message relationships\n");

    // Add edges based on recent events showing message sending
    let mut edges = HashMap::new();
    for (id, metrics) in &supervision.actors {
        for event in &metrics.recent_events {
            if let crate::supervision::EventType::MessageSent { recipient_id } = &event.event_type {
                let edge_key = (id.clone(), recipient_id.clone());
                *edges.entry(edge_key).or_insert(0) += 1;
            }
        }
    }

    for ((from, to), count) in edges.iter() {
        dot.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{} msgs\"];\n",
            from, to, count
        ));
    }

    dot.push_str("}\n");
    dot
}

/// Export actor system supervision data to Mermaid diagram format
pub fn to_mermaid(supervision: &SupervisionData) -> String {
    let mut mermaid = String::from("graph LR\n");

    // Add actor nodes
    for (id, metrics) in &supervision.actors {
        let status_emoji = match metrics.status {
            ActorStatus::Running => "🟢",
            ActorStatus::Stopped => "🟡",
            ActorStatus::Failed => "🔴",
        };
        let label = format!(
            "{}[{}]<br/>{}<br/>Msgs: {}",
            status_emoji, id, metrics.actor_type, metrics.messages_processed
        );
        mermaid.push_str(&format!("  {}[\"{}\"]\n", id.replace('-', "_"), label));
    }

    mermaid.push('\n');

    // Add edges based on message sending
    let mut edges = HashMap::new();
    for (id, metrics) in &supervision.actors {
        for event in &metrics.recent_events {
            if let crate::supervision::EventType::MessageSent { recipient_id } = &event.event_type {
                let edge_key = (id.clone(), recipient_id.clone());
                *edges.entry(edge_key).or_insert(0) += 1;
            }
        }
    }

    for ((from, to), count) in edges.iter() {
        mermaid.push_str(&format!(
            "  {} -->|{} msgs| {}\n",
            from.replace('-', "_"),
            count,
            to.replace('-', "_")
        ));
    }

    mermaid
}

/// Export actor system supervision data to Mermaid sequence diagram format
pub fn to_mermaid_sequence(supervision: &SupervisionData) -> String {
    let mut sequence = String::from("sequenceDiagram\n");

    // Add participants
    for (id, metrics) in &supervision.actors {
        let safe_id = id.replace('-', "_");
        sequence.push_str(&format!("    participant {} as {}({})\n", safe_id, metrics.actor_type, id));
    }

    // Collect all message sending events with timestamps
    let mut events: Vec<(u64, String, String)> = Vec::new();
    
    for (sender_id, metrics) in &supervision.actors {
        for event in &metrics.recent_events {
            if let crate::supervision::EventType::MessageSent { recipient_id } = &event.event_type {
                let safe_from = sender_id.replace('-', "_");
                let safe_to = recipient_id.replace('-', "_");
                events.push((event.timestamp, safe_from, safe_to));
            }
        }
    }

    // Sort events by timestamp
    events.sort_by_key(|e| e.0);

    // Add message flows
    if !events.is_empty() {
        sequence.push('\n');
        for (_, from, to) in events {
            sequence.push_str(&format!("    {}->>{}:message\n", from, to));
        }
    }

    sequence
}

/// Get a simple text summary of the actor system
pub fn to_text_summary(supervision: &SupervisionData) -> String {
    let summary = supervision.summary();
    let mut output = String::new();

    output.push_str("=== Actor System Summary ===\n");
    output.push_str(&format!("Timestamp: {}\n\n", supervision.timestamp));
    output.push_str("Actors:\n");
    output.push_str(&format!("  Total: {}\n", summary.total_actors));
    output.push_str(&format!("  Running: {}\n", summary.active_actors));
    output.push_str(&format!("  Stopped: {}\n", summary.stopped_actors));
    output.push_str(&format!("  Failed: {}\n\n", summary.failed_actors));
    output.push_str("Messages:\n");
    output.push_str(&format!("  Processed: {}\n", summary.total_messages_processed));
    output.push_str(&format!("  Sent: {}\n\n", summary.total_messages_sent));

    output.push_str("Actor Details:\n");
    for (id, metrics) in &supervision.actors {
        output.push_str(&format!("  {} ({})\n", id, metrics.actor_type));
        output.push_str(&format!("    Status: {:?}\n", metrics.status));
        output.push_str(&format!("    Uptime: {} ms\n", metrics.uptime_ms()));
        output.push_str(&format!("    Messages Processed: {}\n", metrics.messages_processed));
        output.push_str(&format!("    Messages Sent: {}\n", metrics.messages_sent));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::supervision::ActorMetrics;

    fn create_test_supervision() -> SupervisionData {
        let mut supervision = SupervisionData::new();
        let mut actor1 = ActorMetrics::new("actor-1".to_string(), "TestActor".to_string());
        actor1.record_message_processed();
        actor1.record_message_sent("actor-2".to_string());

        let actor2 = ActorMetrics::new("actor-2".to_string(), "TestActor".to_string());

        supervision.actors.insert("actor-1".to_string(), actor1);
        supervision.actors.insert("actor-2".to_string(), actor2);
        supervision.total_messages_processed = 1;
        supervision.total_messages_sent = 1;

        supervision
    }

    #[test]
    fn test_to_json_export() {
        let supervision = create_test_supervision();
        let json = to_json(&supervision).expect("Failed to export to JSON");
        assert!(json.contains("\"actor-1\""));
        assert!(json.contains("TestActor"));
        assert!(json.contains("Running"));
    }

    #[test]
    fn test_to_dot_export() {
        let supervision = create_test_supervision();
        let dot = to_dot(&supervision);
        assert!(dot.contains("digraph ActorSystem"));
        assert!(dot.contains("actor-1"));
        assert!(dot.contains("actor-2"));
    }

    #[test]
    fn test_to_mermaid_export() {
        let supervision = create_test_supervision();
        let mermaid = to_mermaid(&supervision);
        assert!(mermaid.contains("graph LR"));
        assert!(mermaid.contains("TestActor"));
    }

    #[test]
    fn test_to_text_summary() {
        let supervision = create_test_supervision();
        let text = to_text_summary(&supervision);
        assert!(text.contains("Actor System Summary"));
        assert!(text.contains("Running: 2"));
        assert!(text.contains("actor-1"));
    }
}
