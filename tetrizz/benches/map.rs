use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use utils::{
    data::{Board, Piece, Rotation},
    movegen::CollisionMap
};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("collision map old", |b| b.iter(|| {
        let board = black_box(Board { cols: black_box([127,63,27,8,1,11,3,3,7,15]) });
        black_box(CollisionMap::new(&board, black_box(Piece::Z), black_box(Rotation::North)))
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);