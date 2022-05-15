use criterion::{black_box, criterion_group, criterion_main, Criterion};
use noak::mutf8::MStr;

pub fn validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation");
    group.bench_function("ascii", |b| {
        b.iter(|| {
            let data = black_box(include_bytes!("../data/mutf8/validation_ascii"));
            assert!(MStr::from_bytes(data).is_ok())
        });
    });
    group.finish();
}

criterion_group!(benches, validation);
criterion_main!(benches);
