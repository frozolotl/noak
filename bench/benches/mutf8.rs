use criterion::{black_box, criterion_group, criterion_main, Criterion};
use noak::mutf8::MStr;

pub fn validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation");
    group.bench_function("ascii", |b| {
        let data = black_box(include_bytes!("../data/mutf8/validation_ascii"));
        b.iter(|| assert!(MStr::from_bytes(data).is_ok()));
    });
    group.bench_function("utf8", |b| {
        let data = black_box(include_bytes!("../data/mutf8/validation_utf8"));
        b.iter(|| assert!(MStr::from_bytes(data).is_ok()));
    });
    group.finish();
}

criterion_group!(benches, validation);
criterion_main!(benches);
