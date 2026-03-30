use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

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
        b.iter(|| {
            sangha::population::logistic_growth(black_box(500.0), black_box(0.5), black_box(1000.0))
        })
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

fn bench_deffuant_interaction(c: &mut Criterion) {
    c.bench_function("opinion/deffuant_interaction", |b| {
        b.iter(|| {
            sangha::opinion::deffuant_interaction(
                black_box(0.3),
                black_box(0.7),
                black_box(0.5),
                black_box(0.5),
            )
        })
    });
}

fn bench_echo_chamber_index(c: &mut Criterion) {
    let opinions: Vec<f64> = (0..100).map(|i| i as f64 / 100.0).collect();
    c.bench_function("opinion/echo_chamber_index_100", |b| {
        b.iter(|| sangha::opinion::echo_chamber_index(black_box(&opinions)))
    });
}

fn bench_conformity_threshold(c: &mut Criterion) {
    c.bench_function("influence/conformity_threshold", |b| {
        b.iter(|| {
            sangha::influence::conformity_threshold(black_box(0.5), black_box(0.7), black_box(5))
        })
    });
}

fn bench_bass_diffusion(c: &mut Criterion) {
    c.bench_function("influence/bass_diffusion", |b| {
        b.iter(|| {
            sangha::influence::bass_diffusion(
                black_box(200),
                black_box(1000),
                black_box(0.03),
                black_box(0.38),
            )
        })
    });
}

fn bench_social_loafing(c: &mut Criterion) {
    c.bench_function("group/social_loafing", |b| {
        b.iter(|| sangha::group::social_loafing(black_box(10), black_box(100.0), black_box(0.1)))
    });
}

criterion_group!(
    benches,
    bench_gini,
    bench_sir_step,
    bench_logistic_growth,
    bench_nash_equilibria,
    bench_clustering_coefficient,
    bench_deffuant_interaction,
    bench_echo_chamber_index,
    bench_conformity_threshold,
    bench_bass_diffusion,
    bench_social_loafing,
);
criterion_main!(benches);
