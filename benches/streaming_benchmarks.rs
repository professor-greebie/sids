use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sids::actors::start_actor_system;
use sids::streaming::flow::Flow;
use sids::streaming::sink::Sink;
use sids::streaming::source::Source;
use sids::streaming::stream_message::{NotUsed, StreamMessage};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

fn bench_source_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("source_single_item", |b| {
        b.to_async(&rt).iter(|| async {
            let actor_system = start_actor_system::<StreamMessage, StreamMessage>();
            let source = Source::new("test".to_string(), NotUsed);
            black_box(source);
            black_box(actor_system);
        });
    });

    let mut group = c.benchmark_group("source_from_items");
    for count in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.to_async(&rt).iter(|| async move {
                let actor_system = start_actor_system::<StreamMessage, StreamMessage>();
                let items: Vec<String> = (0..count).map(|i| format!("item{}", i)).collect();
                let source = Source::from_items(items);
                black_box(source);
                black_box(actor_system);
            });
        });
    }
    group.finish();
}

fn bench_source_to_sink(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("source_to_sink_simple", |b| {
        b.to_async(&rt).iter(|| async {
            let mut actor_system = start_actor_system::<StreamMessage, StreamMessage>();
            let source = Source::new("test".to_string(), NotUsed);
            let sink = Sink::new("sink".to_string(), |_msg: StreamMessage| {});
            let _materializer = source.to_sink(&mut actor_system, sink).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            black_box(actor_system);
        });
    });

    let mut group = c.benchmark_group("source_to_sink_throughput");
    for count in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.to_async(&rt).iter(|| async move {
                let mut actor_system = start_actor_system::<StreamMessage, StreamMessage>();
                let items: Vec<String> = (0..count).map(|i| format!("item{}", i)).collect();
                let source = Source::from_items(items);

                let counter = Arc::new(Mutex::new(0));
                let counter_clone = counter.clone();
                let sink = Sink::new("sink".to_string(), move |msg: StreamMessage| {
                    if let StreamMessage::Text(_) = msg {
                        let mut count = counter_clone.lock().unwrap();
                        *count += 1;
                    }
                });

                let _materializer = source.to_sink(&mut actor_system, sink).await;
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
                black_box(actor_system);
            });
        });
    }
    group.finish();
}

fn bench_source_via_flow_to_sink(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("source_flow_sink", |b| {
        b.to_async(&rt).iter(|| async {
            let mut actor_system = start_actor_system::<StreamMessage, StreamMessage>();
            let source = Source::new("test".to_string(), NotUsed);
            let flow = Flow::new("uppercase".to_string(), |msg: StreamMessage| match msg {
                StreamMessage::Text(s) => StreamMessage::Text(s.to_uppercase()),
                other => other,
            });
            let sink = Sink::new("sink".to_string(), |_msg: StreamMessage| {});
            let _materializer = source.via_to_sink(&mut actor_system, flow, sink).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            black_box(actor_system);
        });
    });

    let mut group = c.benchmark_group("flow_throughput");
    for count in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.to_async(&rt).iter(|| async move {
                let mut actor_system = start_actor_system::<StreamMessage, StreamMessage>();
                let items: Vec<String> = (0..count).map(|i| format!("item{}", i)).collect();
                let source = Source::from_items(items);

                let flow = Flow::new("uppercase".to_string(), |msg: StreamMessage| match msg {
                    StreamMessage::Text(s) => StreamMessage::Text(s.to_uppercase()),
                    other => other,
                });

                let counter = Arc::new(Mutex::new(0));
                let counter_clone = counter.clone();
                let sink = Sink::new("counter".to_string(), move |msg: StreamMessage| {
                    if let StreamMessage::Text(_) = msg {
                        let mut count = counter_clone.lock().unwrap();
                        *count += 1;
                    }
                });

                let _materializer = source.via_to_sink(&mut actor_system, flow, sink).await;
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
                black_box(actor_system);
            });
        });
    }
    group.finish();
}

fn bench_source_map_operations(c: &mut Criterion) {
    c.bench_function("source_map", |b| {
        b.iter(|| {
            let source = Source::new("test".to_string(), NotUsed);
            let mapped = source.map(|s| s.to_uppercase());
            black_box(mapped);
        });
    });

    c.bench_function("source_filter", |b| {
        b.iter(|| {
            let source = Source::new("test string with data".to_string(), NotUsed);
            let filtered = source.filter(|s| s.len() > 5);
            black_box(filtered);
        });
    });

    c.bench_function("source_map_filter", |b| {
        b.iter(|| {
            let source = Source::new("test".to_string(), NotUsed);
            let processed = source.map(|s| s.to_uppercase()).filter(|s| s.len() > 2);
            black_box(processed);
        });
    });
}

criterion_group!(
    benches,
    bench_source_creation,
    bench_source_to_sink,
    bench_source_via_flow_to_sink,
    bench_source_map_operations
);
criterion_main!(benches);
