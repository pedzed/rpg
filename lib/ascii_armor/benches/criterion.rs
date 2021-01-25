use std::io::Cursor;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ascii_armor::ArmorReader;

pub fn criterion_benchmark(c: &mut Criterion) {
    let buffer = Cursor::new(b"A".repeat(5000));

    c.bench_function("ArmorReader::read", |b| b.iter(|| {
        let mut buffer = buffer.clone();
        let _armor = ArmorReader::read(black_box(&mut buffer)).unwrap();
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
