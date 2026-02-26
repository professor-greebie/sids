use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sids::actors::{
    actor::Actor,
    get_response_handler,
    messages::{Message, ResponseMessage},
    send_message_by_id, spawn_actor, start_actor_system,
};
use tokio::runtime::Runtime;

// Simple actor for benchmarking
struct BenchActor {
    counter: u64,
}

impl Actor<String, ResponseMessage> for BenchActor {
    async fn receive(&mut self, message: Message<String, ResponseMessage>) {
        if let Some(_payload) = message.payload {
            self.counter += 1;
        }
        if let Some(responder) = message.responder {
            responder.handle(ResponseMessage::Success).await;
        }
    }
}

// Empty actor for minimal overhead benchmarking
struct EmptyActor;

impl Actor<String, ResponseMessage> for EmptyActor {
    async fn receive(&mut self, _message: Message<String, ResponseMessage>) {
        // Minimal work
    }
}

fn bench_actor_spawn(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("actor_spawn_single", |b| {
        b.to_async(&rt).iter(|| async {
            let mut actor_system = start_actor_system::<String, ResponseMessage>();
            spawn_actor(
                &mut actor_system,
                BenchActor { counter: 0 },
                Some("BenchActor".to_string()),
            )
            .await;
            black_box(actor_system);
        });
    });

    let mut group = c.benchmark_group("actor_spawn_bulk");
    for count in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.to_async(&rt).iter(|| async move {
                let mut actor_system = start_actor_system::<String, ResponseMessage>();
                for i in 0..count {
                    spawn_actor(
                        &mut actor_system,
                        BenchActor { counter: 0 },
                        Some(format!("Actor{}", i)),
                    )
                    .await;
                }
                black_box(actor_system);
            });
        });
    }
    group.finish();
}

fn bench_message_passing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("message_send_no_response", |b| {
        b.to_async(&rt).iter(|| async {
            let mut actor_system = start_actor_system::<String, ResponseMessage>();
            spawn_actor(
                &mut actor_system,
                EmptyActor,
                Some("EmptyActor".to_string()),
            )
            .await;

            let msg = Message {
                payload: Some("test".to_string()),
                stop: false,
                responder: None,
                blocking: None,
            };

            send_message_by_id(&mut actor_system, 0, msg).await.unwrap();
            black_box(actor_system);
        });
    });

    c.bench_function("message_send_with_response", |b| {
        b.to_async(&rt).iter(|| async {
            let mut actor_system = start_actor_system::<String, ResponseMessage>();
            spawn_actor(
                &mut actor_system,
                BenchActor { counter: 0 },
                Some("BenchActor".to_string()),
            )
            .await;

            let (handler, rx) = get_response_handler::<ResponseMessage>();
            let msg = Message {
                payload: Some("test".to_string()),
                stop: false,
                responder: Some(handler),
                blocking: None,
            };

            send_message_by_id(&mut actor_system, 0, msg).await.unwrap();
            let _response = rx.await.unwrap();
            black_box(actor_system);
        });
    });

    let mut group = c.benchmark_group("message_throughput");
    for count in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.to_async(&rt).iter(|| async move {
                let mut actor_system = start_actor_system::<String, ResponseMessage>();
                spawn_actor(
                    &mut actor_system,
                    EmptyActor,
                    Some("EmptyActor".to_string()),
                )
                .await;

                for _ in 0..count {
                    let msg = Message {
                        payload: Some("test".to_string()),
                        stop: false,
                        responder: None,
                        blocking: None,
                    };
                    send_message_by_id(&mut actor_system, 0, msg).await.unwrap();
                }
                black_box(actor_system);
            });
        });
    }
    group.finish();
}

fn bench_concurrent_actors(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_message_passing");
    for actor_count in [5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(actor_count),
            actor_count,
            |b, &actor_count| {
                b.to_async(&rt).iter(|| async move {
                    let mut actor_system = start_actor_system::<String, ResponseMessage>();

                    // Spawn multiple actors
                    for i in 0..actor_count {
                        spawn_actor(
                            &mut actor_system,
                            BenchActor { counter: 0 },
                            Some(format!("Actor{}", i)),
                        )
                        .await;
                    }

                    // Send messages to all actors
                    let mut handles = Vec::new();
                    for i in 0..actor_count {
                        let msg = Message {
                            payload: Some("test".to_string()),
                            stop: false,
                            responder: None,
                            blocking: None,
                        };
                        let result = send_message_by_id(&mut actor_system, i as u32, msg).await;
                        handles.push(result);
                    }

                    black_box(actor_system);
                });
            },
        );
    }
    group.finish();
}

fn bench_response_handler(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("response_handler_creation", |b| {
        b.iter(|| {
            let (handler, _rx) = get_response_handler::<ResponseMessage>();
            black_box(handler);
        });
    });

    c.bench_function("response_handler_roundtrip", |b| {
        b.to_async(&rt).iter(|| async {
            let (handler, rx) = get_response_handler::<ResponseMessage>();
            handler.handle(ResponseMessage::Success).await;
            let _response = rx.await.unwrap();
        });
    });

    let mut group = c.benchmark_group("response_handler_batch");
    for count in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.to_async(&rt).iter(|| async move {
                let mut handlers = Vec::new();
                let mut receivers = Vec::new();

                for _ in 0..count {
                    let (handler, rx) = get_response_handler::<ResponseMessage>();
                    handlers.push(handler);
                    receivers.push(rx);
                }

                for handler in handlers {
                    handler.handle(ResponseMessage::Success).await;
                }

                for rx in receivers {
                    let _response = rx.await.unwrap();
                }
            });
        });
    }
    group.finish();
}

fn bench_actor_lookup(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("actor_lookup");
    for actor_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(actor_count),
            actor_count,
            |b, &actor_count| {
                b.to_async(&rt).iter(|| async move {
                    let mut actor_system = start_actor_system::<String, ResponseMessage>();

                    // Spawn actors
                    for i in 0..actor_count {
                        spawn_actor(&mut actor_system, EmptyActor, Some(format!("Actor{}", i)))
                            .await;
                    }

                    // Lookup actors by name
                    for i in 0..actor_count {
                        let _id =
                            sids::actors::find_actor_by_name(&actor_system, &format!("Actor{}", i))
                                .unwrap();
                    }

                    black_box(actor_system);
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_actor_spawn,
    bench_message_passing,
    bench_concurrent_actors,
    bench_response_handler,
    bench_actor_lookup
);
criterion_main!(benches);
