use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_gini(c: &mut Criterion) {
    let incomes: Vec<f64> = (1..=100).map(|x| x as f64).collect();
    c.bench_function("inequality/gini_coefficient_100", |b| {
        b.iter(|| sangha::inequality::gini_coefficient(black_box(&incomes)))
    });
}

fn bench_sir_step(c: &mut Criterion) {
    c.bench_function("population/sir_step", |b| {
        b.iter(|| {
            sangha::population::sir_step(
                black_box(0.99),
                black_box(0.01),
                black_box(0.0),
                black_box(0.5),
                black_box(0.1),
                black_box(0.01),
            )
        })
    });
}

fn bench_logistic_growth(c: &mut Criterion) {
    c.bench_function("population/logistic_growth", |b| {
        b.iter(|| sangha::population::logistic_growth(black_box(500.0), black_box(0.5), black_box(1000.0)))
    });
}

fn bench_nash_equilibria(c: &mut Criterion) {
    let pd = sangha::game_theory::prisoners_dilemma();
    c.bench_function("game_theory/find_nash_equilibria", |b| {
        b.iter(|| sangha::game_theory::find_nash_equilibria(black_box(&pd)))
    });
}

fn bench_clustering_coefficient(c: &mut Criterion) {
    let net = sangha::network::watts_strogatz(50, 4, 0.0).unwrap();
    c.bench_function("network/clustering_coefficient", |b| {
        b.iter(|| sangha::network::clustering_coefficient(black_box(&net), black_box(0)))
    });
}

criterion_group!(
    benches,
    bench_gini,
    bench_sir_step,
    bench_logistic_growth,
    bench_nash_equilibria,
    bench_clustering_coefficient,
);
criterion_main!(benches);
