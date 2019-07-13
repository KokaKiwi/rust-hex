use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rustc_hex::{FromHex, ToHex};

const SMALL_DATA: &[u8] =
    b"Day before yesterday I saw a rabbit, and yesterday a deer, and today, you.";
const BIG_DATA: &[u8] = include_bytes!("../src/lib.rs");

fn bench_small_data(c: &mut Criterion) {
    c.bench_function("hex_encode_small", |b| {
        b.iter(|| black_box(hex::encode(SMALL_DATA)))
    });

    c.bench_function("rustc_hex_encode_small", |b| {
        b.iter(|| black_box(SMALL_DATA.to_hex::<String>()))
    });

    c.bench_function("hex_decode_small", |b| {
        let hex = hex::encode(SMALL_DATA);
        b.iter(|| black_box(hex::decode(&hex).unwrap()))
    });

    c.bench_function("rustc_hex_decode_small", |b| {
        let hex = hex::encode(SMALL_DATA);
        b.iter(|| black_box(hex.from_hex::<Vec<u8>>().unwrap()))
    });
}

fn bench_big_data(c: &mut Criterion) {
    c.bench_function("hex_encode_big", |b| {
        b.iter(|| black_box(hex::encode(BIG_DATA)))
    });

    c.bench_function("rustc_hex_encode_big", |b| {
        b.iter(|| black_box(BIG_DATA.to_hex::<String>()))
    });

    c.bench_function("hex_decode_big", |b| {
        let hex = hex::encode(BIG_DATA);
        b.iter(|| black_box(hex::decode(&hex).unwrap()))
    });

    c.bench_function("rustc_hex_decode_big", |b| {
        let hex = hex::encode(BIG_DATA);
        b.iter(|| black_box(hex.from_hex::<Vec<u8>>().unwrap()))
    });
}

criterion_group!(benches, bench_small_data, bench_big_data);
criterion_main!(benches);
