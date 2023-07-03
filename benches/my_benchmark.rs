use criterion::{criterion_group, criterion_main, Criterion};

fn run(c: &mut Criterion) {
    let mut group = c.benchmark_group("run");
    group.sample_size(10);
    group.bench_function("run", |b| {
        b.iter(|| {
            // Always use the same config for benchmarking
            let config = aharc::config::Config {
                comments: "input/ah_comments.json".into(),
                submissions: "input/ah_posts.json".into(),
                limit_posts: Some(500),
            };
            aharc::run(config).unwrap()
        });
    });
    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
