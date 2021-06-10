use criterion::BenchmarkId;
use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Criterion};
use intelhexes::hex2dump;

fn nrf_bench(c: &mut Criterion) {
    let hex = include_bytes!("../hex-examples/sniffer_nrf52840dk_nrf52840_7cc811f.hex");
    let null = std::fs::File::create("/dev/null").expect("to be able to open /dev/null");
    let mut group = c.benchmark_group("NRF");
    group.throughput(Throughput::Bytes(hex.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("hex2dump", hex.len()),
        &null,
        |b, file| {
            b.iter(|| {
                hex2dump(std::io::Cursor::new(hex), file).expect("to be able to parse the hex")
            })
        },
    );
    group.finish();
}

fn nina_bench(c: &mut Criterion) {
    let hex = include_bytes!("../hex-examples/NINA-W15X-SW-4.0.0-006.hex");
    let null = std::fs::File::create("/dev/null").expect("to be able to open /dev/null");
    let mut group = c.benchmark_group("NINA");
    group.throughput(Throughput::Bytes(hex.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("hex2dump", hex.len()),
        &null,
        |b, file| {
            b.iter(|| {
                hex2dump(std::io::Cursor::new(hex), file).expect("to be able to parse the hex")
            })
        },
    );
    group.finish();
}

criterion_group!(benches, nrf_bench, nina_bench);
criterion_main!(benches);
