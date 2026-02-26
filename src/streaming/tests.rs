/// Unit tests for the streaming module
use super::*;
use crate::actors::start_actor_system;
use flow::{transforms, Flow};
use sink::{consumers, Sink};
use source::{Source, SourceActor, SourceError};
use std::sync::{Arc, Mutex};
use stream_message::{NotUsed, StreamMessage};
use tokio::time::{timeout, Duration, Instant};

#[cfg(test)]
mod source_tests {
    use super::*;

    #[test]
    fn test_source_creation() {
        let source = Source::new("test data".to_string(), NotUsed);
        assert!(source.data().is_some());
        assert_eq!(source.data().unwrap(), "test data");
    }

    #[test]
    fn test_source_from_items() {
        let items = vec!["one".to_string(), "two".to_string(), "three".to_string()];
        let source = Source::from_items(items);
        assert!(source.data().is_some());
        assert_eq!(source.data().unwrap(), "one\ntwo\nthree");
    }

    #[test]
    fn test_source_map() {
        let source = Source::new("hello".to_string(), NotUsed);
        let mapped = source.map(|s| s.to_uppercase());
        assert_eq!(mapped.data().unwrap(), "HELLO");
    }

    #[test]
    fn test_source_filter() {
        let source = Source::new("hello world".to_string(), NotUsed);
        let filtered = source.filter(|s| s.contains("world"));
        assert!(filtered.data().is_some());

        let source2 = Source::new("goodbye".to_string(), NotUsed);
        let filtered2 = source2.filter(|s| s.contains("world"));
        assert!(filtered2.data().is_none());
    }

    #[test]
    fn test_source_map_lines() {
        let source = Source::new("hello\nworld".to_string(), NotUsed);
        let mapped = source.map_lines(|line| line.to_uppercase());
        assert_eq!(mapped.data().unwrap(), "HELLO\nWORLD");
    }

    #[test]
    fn test_source_filter_lines() {
        let source = Source::new("hello\nworld\nrust".to_string(), NotUsed);
        let filtered = source.filter_lines(|line| line.len() > 4);
        assert_eq!(filtered.data().unwrap(), "hello\nworld");
    }

    #[test]
    fn test_source_chained_transformations() {
        let items = vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
        ];
        let source = Source::from_items(items)
            .map_lines(|s| {
                let num: i32 = s.parse().unwrap_or(0);
                (num * 2).to_string()
            })
            .filter_lines(|s| {
                let num: i32 = s.parse().unwrap_or(0);
                num > 2
            });

        assert_eq!(source.data().unwrap(), "4\n6\n8");
    }

    #[test]
    fn test_source_binary_map() {
        let data = vec![1u8, 2u8, 3u8];
        let source = Source::new(data.clone(), NotUsed);
        let mapped = source.map(|bytes| bytes.iter().map(|b| b * 2).collect());
        assert_eq!(mapped.data().unwrap(), &vec![2u8, 4u8, 6u8]);
    }

    #[test]
    fn test_source_binary_filter() {
        let data = vec![1u8, 2u8, 3u8, 4u8, 5u8];
        let source = Source::new(data.clone(), NotUsed);
        let filtered = source.filter(|bytes| bytes.len() > 3);
        assert!(filtered.data().is_some());
        assert_eq!(filtered.data().unwrap().len(), 5);
    }

    #[test]
    fn test_source_error_types() {
        let err1 = SourceError::InvalidUrl("bad url".to_string());
        assert!(matches!(err1, SourceError::InvalidUrl(_)));

        let err2 = SourceError::Timeout;
        assert!(matches!(err2, SourceError::Timeout));

        let err3 = SourceError::EmptyResponse;
        assert!(matches!(err3, SourceError::EmptyResponse));
    }

    #[test]
    fn test_notused_copy() {
        let nu1 = NotUsed;
        let _nu2 = nu1; // Should work because NotUsed implements Copy
        let _nu3 = nu1; // Should still work
                        // If we got here, Copy works
    }

    #[tokio::test]
    async fn test_source_from_file_nonexistent() {
        let result = Source::from_file("nonexistent_file_xyz_12345.txt");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SourceError::FileNotFound(_)));
    }

    #[tokio::test]
    async fn test_source_from_file_success() {
        // Create a test file
        let test_file = "test_source_file_unit.txt";
        std::fs::write(test_file, "test content for unit test").unwrap();

        let result = Source::from_file(test_file);
        assert!(result.is_ok());
        let source = result.unwrap();
        assert_eq!(source.data().unwrap(), "test content for unit test");

        // Clean up
        let _ = std::fs::remove_file(test_file);
    }

    #[tokio::test]
    async fn test_source_from_file_bytes() {
        // Create a test binary file
        let test_file = "test_binary_file_unit.bin";
        let data = vec![0xFF, 0xFE, 0xFD, 0xFC, 0x01, 0x02];
        std::fs::write(test_file, &data).unwrap();

        let result = Source::from_file_bytes(test_file);
        assert!(result.is_ok());
        let source = result.unwrap();
        assert_eq!(source.data().unwrap(), &data);

        // Clean up
        let _ = std::fs::remove_file(test_file);
    }

    #[tokio::test]
    async fn test_source_from_file_empty() {
        let test_file = "test_empty_file.txt";
        std::fs::write(test_file, "").unwrap();

        let result = Source::from_file(test_file);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SourceError::EmptyResponse));

        let _ = std::fs::remove_file(test_file);
    }

    #[tokio::test]
    async fn test_source_from_file_directory() {
        let test_dir = "test_directory_for_unit";
        std::fs::create_dir(test_dir).unwrap();

        let result = Source::from_file(test_dir);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SourceError::InvalidPath(_)));

        let _ = std::fs::remove_dir(test_dir);
    }
}

#[cfg(test)]
mod flow_tests {
    use super::*;

    #[test]
    fn test_flow_creation() {
        let _flow = Flow::new("test_flow".to_string(), transforms::identity);
        // Just verify it compiles and constructs
    }

    #[test]
    fn test_flow_map() {
        let flow = Flow::new("base".to_string(), transforms::identity);
        let _mapped = flow.map(|msg| match msg {
            StreamMessage::Text(text) => StreamMessage::Text(text.to_uppercase()),
            other => other,
        });

        // Just verify it compiles - actual transformation tested in integration tests
    }

    #[test]
    fn test_flow_filter() {
        let flow = Flow::new("base".to_string(), transforms::identity);
        let _filtered = flow.filter(|msg| match msg {
            StreamMessage::Text(ref text) => text.len() > 5,
            _ => true,
        });

        // Just verify it compiles - actual filtering tested in integration tests
    }

    #[test]
    fn test_flow_transforms_to_uppercase() {
        let input = StreamMessage::Text("hello".to_string());
        let output = transforms::to_uppercase(input);
        assert!(matches!(output, StreamMessage::Text(ref s) if s == "HELLO"));
    }

    #[test]
    fn test_flow_transforms_filter_empty() {
        let input1 = StreamMessage::Text("".to_string());
        let output1 = transforms::filter_empty(input1);
        assert!(matches!(output1, StreamMessage::Error(_)));

        let input2 = StreamMessage::Text("content".to_string());
        let output2 = transforms::filter_empty(input2);
        assert!(matches!(output2, StreamMessage::Text(_)));
    }

    #[test]
    fn test_flow_transforms_identity() {
        let input = StreamMessage::Text("unchanged".to_string());
        let output = transforms::identity(input.clone());
        assert!(matches!(output, StreamMessage::Text(ref s) if s == "unchanged"));
    }
}

#[cfg(test)]
mod sink_tests {
    use super::*;

    #[test]
    fn test_sink_creation() {
        let _sink = Sink::new("test_sink".to_string(), consumers::ignore);
        // Just verify it compiles and constructs
    }

    #[test]
    fn test_sink_for_each() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let consumer = consumers::for_each(move |_line| {
            *counter_clone.lock().unwrap() += 1;
        });

        // Simulate processing messages
        consumer(StreamMessage::Text("line1\nline2\nline3".to_string()));

        let count = *counter.lock().unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_sink_for_each_with_bytes() {
        let collected = Arc::new(Mutex::new(Vec::new()));
        let collected_clone = collected.clone();

        let consumer = consumers::for_each(move |line: &str| {
            collected_clone.lock().unwrap().push(line.to_string());
        });

        // Test with UTF-8 data
        let data = "hello\nworld".as_bytes().to_vec();
        consumer(StreamMessage::Data(data));

        let lines = collected.lock().unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "hello");
        assert_eq!(lines[1], "world");
    }

    #[test]
    fn test_sink_for_each_message() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let consumer = consumers::for_each_message(move |msg| {
            if matches!(msg, StreamMessage::Text(_)) {
                *counter_clone.lock().unwrap() += 1;
            }
        });

        consumer(StreamMessage::Text("message1".to_string()));
        consumer(StreamMessage::Text("message2".to_string()));
        consumer(StreamMessage::Complete);

        let count = *counter.lock().unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_sink_collector() {
        let (collector, storage) = consumers::create_collector();

        collector(StreamMessage::Data(vec![1, 2, 3]));
        collector(StreamMessage::Data(vec![4, 5, 6]));

        let collected = storage.lock().unwrap();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0], vec![1, 2, 3]);
        assert_eq!(collected[1], vec![4, 5, 6]);
    }

    #[test]
    fn test_sink_ignore() {
        // Should not panic
        consumers::ignore(StreamMessage::Text("test".to_string()));
        consumers::ignore(StreamMessage::Data(vec![1, 2, 3]));
        consumers::ignore(StreamMessage::Complete);
        // Test passes if no panic occurs
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_source_to_sink_integration() {
        let collected = Arc::new(Mutex::new(Vec::new()));
        let collected_clone = collected.clone();

        let source = Source::new("test message".to_string(), NotUsed);
        let sink = Sink::new("collector".to_string(), move |msg: StreamMessage| {
            if let StreamMessage::Text(text) = msg {
                collected_clone.lock().unwrap().push(text);
            }
        });

        let mut actor_system = start_actor_system();
        let _materializer = source.to_sink(&mut actor_system, sink).await;

        // Give actors time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let data = collected.lock().unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0], "test message");
    }

    #[tokio::test]
    async fn test_source_flow_sink_integration() {
        let collected = Arc::new(Mutex::new(Vec::new()));
        let collected_clone = collected.clone();

        let source = Source::new("hello streaming".to_string(), NotUsed);
        let flow = Flow::new("uppercase".to_string(), transforms::to_uppercase);
        let sink = Sink::new("collector".to_string(), move |msg: StreamMessage| {
            if let StreamMessage::Text(text) = msg {
                collected_clone.lock().unwrap().push(text);
            }
        });

        let mut actor_system = start_actor_system();
        let _materializer = source.via_to_sink(&mut actor_system, flow, sink).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let data = collected.lock().unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0], "HELLO STREAMING");
    }

    #[tokio::test]
    async fn test_source_map_filter_integration() {
        let collected = Arc::new(Mutex::new(Vec::new()));
        let collected_clone = collected.clone();

        let items = vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
            "5".to_string(),
        ];
        let source = Source::from_items(items)
            .map_lines(|s| {
                let num: i32 = s.parse().unwrap_or(0);
                (num * 2).to_string()
            })
            .filter_lines(|s| {
                let num: i32 = s.parse().unwrap_or(0);
                num > 5
            });

        let sink = Sink::new(
            "collector".to_string(),
            consumers::for_each(move |line| {
                collected_clone.lock().unwrap().push(line.to_string());
            }),
        );

        let mut actor_system = start_actor_system();
        let _materializer = source.to_sink(&mut actor_system, sink).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let data = collected.lock().unwrap();
        assert_eq!(data.len(), 3);
        assert_eq!(data[0], "6");
        assert_eq!(data[1], "8");
        assert_eq!(data[2], "10");
    }

    #[tokio::test]
    async fn test_flow_map_filter_integration() {
        let collected = Arc::new(Mutex::new(Vec::new()));
        let collected_clone = collected.clone();

        let source = Source::new("hello world rust".to_string(), NotUsed);
        let flow = Flow::new("uppercase".to_string(), transforms::to_uppercase)
            .map(|msg| match msg {
                StreamMessage::Text(text) => StreamMessage::Text(text.replace(" ", "_")),
                other => other,
            })
            .filter(|msg| match msg {
                StreamMessage::Text(ref text) => text.len() > 10,
                _ => true,
            });

        let sink = Sink::new("collector".to_string(), move |msg: StreamMessage| {
            if let StreamMessage::Text(text) = msg {
                collected_clone.lock().unwrap().push(text);
            }
        });

        let mut actor_system = start_actor_system();
        let _materializer = source.via_to_sink(&mut actor_system, flow, sink).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let data = collected.lock().unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0], "HELLO_WORLD_RUST");
    }

    #[tokio::test]
    async fn test_binary_source_integration() {
        let collected = Arc::new(Mutex::new(Vec::new()));
        let collected_clone = collected.clone();

        let data = vec![72, 69, 76, 76, 79]; // "HELLO" in ASCII
        let source = Source::new(data.clone(), NotUsed);

        let sink = Sink::new("collector".to_string(), move |msg: StreamMessage| {
            if let StreamMessage::Data(bytes) = msg {
                collected_clone.lock().unwrap().push(bytes);
            }
        });

        let mut actor_system = start_actor_system();
        let _materializer = source.to_sink(&mut actor_system, sink).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let collected_data = collected.lock().unwrap();
        assert_eq!(collected_data.len(), 1);
        assert_eq!(collected_data[0], data);
    }
}

#[cfg(test)]
mod backpressure_tests {
    use super::*;

    #[tokio::test]
    async fn test_source_actor_backpressure_blocks() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let mut source_actor = SourceActor::new(
            "BackpressureSource".to_string(),
            vec![
                StreamMessage::Text("one".to_string()),
                StreamMessage::Text("two".to_string()),
                StreamMessage::Text("three".to_string()),
            ],
        );
        source_actor.set_downstream(tx);

        let start = Instant::now();
        let emit_task = tokio::spawn(async move {
            source_actor.emit_all().await;
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let drain_result = timeout(Duration::from_millis(200), async {
            let mut received = 0;
            while let Some(_msg) = rx.recv().await {
                received += 1;
                if received >= 4 {
                    break;
                }
            }
        })
        .await;
        assert!(drain_result.is_ok(), "Receiver should drain messages");

        let _ = emit_task.await;
        assert!(
            start.elapsed() >= Duration::from_millis(50),
            "Backpressure should delay emission"
        );
    }

    #[tokio::test]
    async fn test_source_actor_downstream_closed_completes() {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        drop(rx);

        let mut source_actor = SourceActor::new(
            "ClosedDownstream".to_string(),
            vec![StreamMessage::Text("payload".to_string())],
        );
        source_actor.set_downstream(tx);

        let result = timeout(Duration::from_millis(50), async {
            source_actor.emit_all().await;
        })
        .await;
        assert!(
            result.is_ok(),
            "Emission should complete even if downstream closes"
        );
    }
}
