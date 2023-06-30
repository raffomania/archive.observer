use criterion::{criterion_group, criterion_main, Criterion};

fn run(c: &mut Criterion) {
    let mut group = c.benchmark_group("run");
    group.sample_size(20);
    group.bench_function("run", |b| {
        b.iter(|| aharc::run().unwrap());
    });
    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
