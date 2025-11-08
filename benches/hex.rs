use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use rustc_hex::{FromHex, ToHex};

const DATA: &[u8] = include_bytes!("../src/lib.rs");

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");
    group.throughput(Throughput::Bytes(DATA.len() as u64));

    group.bench_function("hex", |b| b.iter(|| hex::encode(DATA)));

    group.bench_function("rustc_hex", |b| b.iter(|| DATA.to_hex::<String>()));

    group.bench_function("faster_hex", |b| b.iter(|| faster_hex::hex_string(DATA)));

    group.bench_function("faster_hex/fallback", |b| {
        b.iter(|| {
            let mut dst = vec![0; DATA.len() * 2];
            faster_hex::hex_encode_fallback(DATA, &mut dst);
            dst
        })
    });

    group.bench_function("data_encoding", |b| {
        b.iter(|| data_encoding::HEXLOWER.encode(DATA))
    });

    group.finish()
}

fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");
    group.throughput(Throughput::Bytes(DATA.len() as u64));

    group.bench_function("hex", |b| {
        let hex = hex::encode(DATA);
        b.iter(|| hex::decode(&hex).unwrap())
    });

    group.bench_function("rustc_hex", |b| {
        let hex = DATA.to_hex::<String>();
        b.iter(|| hex.from_hex::<Vec<u8>>().unwrap())
    });

    group.bench_function("faster_hex", move |b| {
        let hex = faster_hex::hex_string(DATA);
        let len = DATA.len();
        let mut dst = vec![0; len];

        b.iter(|| faster_hex::hex_decode(hex.as_bytes(), &mut dst).unwrap())
    });

    group.bench_function("faster_hex/unchecked", |b| {
        let hex = faster_hex::hex_string(DATA);
        let len = DATA.len();
        let mut dst = vec![0; len];

        b.iter(|| faster_hex::hex_decode_unchecked(hex.as_bytes(), &mut dst))
    });

    group.bench_function("faster_hex/fallback", |b| {
        let hex = faster_hex::hex_string(DATA);
        let len = DATA.len();
        let mut dst = vec![0; len];

        b.iter(|| faster_hex::hex_decode_fallback(hex.as_bytes(), &mut dst))
    });

    group.bench_function("data_encoding", |b| {
        let hex = data_encoding::HEXLOWER.encode(DATA);
        b.iter(|| data_encoding::HEXLOWER.decode(hex.as_bytes()).unwrap())
    });

    group.finish()
}

criterion_group!(benches, bench_encode, bench_decode);
criterion_main!(benches);
