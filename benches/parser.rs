use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nce_qualifiers::parser::Qualifiers;
use std::fs;

// For privacy reasons and I don't get sued from the parents' of the
// PSHS NCE 2024 examinees, I'll make my own sample then.
const QUALIFIER_SAMPLE: &str = concat!(
    "Smith\tJohn\t0000001\t001\t7:30 AM\t1\t0001\tUniversity of the Philippines Diliman\t",
    "Kalaw Corner, Quirino Street, UP Diliman, Quezon City, Metro Manila"
);

fn perform_benchmark(c: &mut Criterion) {
    c.bench_function("parse qualifier line", |b| {
        b.iter(|| {
            Qualifiers::from_str(QUALIFIER_SAMPLE)
                .next()
                .unwrap()
                .unwrap();
        });
    });

    c.bench_function("parse all qualifiers and make an array", |b| {
        let file = fs::read_to_string("qualifiers.txt").unwrap();
        b.iter(|| {
            let iter = Qualifiers::from_str(&file);
            black_box(iter.collect::<Vec<_>>());
        });
    });
}

criterion_group!(benches, perform_benchmark);
criterion_main!(benches);
