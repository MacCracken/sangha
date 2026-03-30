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

fn bench_borda_count(c: &mut Criterion) {
    let ballots: Vec<sangha::collective::RankedBallot> = (0..50)
        .map(|i| {
            sangha::collective::RankedBallot::new(vec![
                i % 5,
                (i + 1) % 5,
                (i + 2) % 5,
                (i + 3) % 5,
                (i + 4) % 5,
            ])
        })
        .collect();
    c.bench_function("collective/borda_count_50v_5c", |b| {
        b.iter(|| sangha::collective::borda_count(black_box(&ballots), black_box(5)))
    });
}

fn bench_jury_theorem(c: &mut Criterion) {
    c.bench_function("collective/jury_theorem_101", |b| {
        b.iter(|| sangha::collective::jury_theorem(black_box(0.7), black_box(101)))
    });
}

fn bench_shapley_value(c: &mut Criterion) {
    // 10-player game: v(S) = |S|^2 (superadditive)
    let values: Vec<f64> = (0..1024)
        .map(|mask: usize| {
            let size = mask.count_ones() as f64;
            size * size
        })
        .collect();
    let game = sangha::coalition::CoalitionGame::new(10, values).unwrap();
    c.bench_function("coalition/shapley_value_10p", |b| {
        b.iter(|| sangha::coalition::shapley_value(black_box(&game)))
    });
}

fn bench_is_core_stable(c: &mut Criterion) {
    let values: Vec<f64> = (0..1024)
        .map(|mask: usize| {
            let size = mask.count_ones() as f64;
            size * size
        })
        .collect();
    let game = sangha::coalition::CoalitionGame::new(10, values).unwrap();
    let alloc = vec![10.0; 10]; // 100 total = v(N) = 10^2
    c.bench_function("coalition/is_core_stable_10p", |b| {
        b.iter(|| sangha::coalition::is_core_stable(black_box(&game), black_box(&alloc)))
    });
}

fn bench_public_goods_round(c: &mut Criterion) {
    let game = sangha::coordination::PublicGoodsGame::new(100, 2.0, 10.0).unwrap();
    let contributions: Vec<f64> = (0..100).map(|i| i as f64 % 11.0).collect();
    c.bench_function("coordination/public_goods_100", |b| {
        b.iter(|| {
            sangha::coordination::public_goods_round(black_box(&game), black_box(&contributions))
        })
    });
}

fn bench_sealed_bid_auction(c: &mut Criterion) {
    let bids: Vec<f64> = (0..100).map(|i| i as f64 * 1.5).collect();
    c.bench_function("coordination/auction_100", |b| {
        b.iter(|| {
            sangha::coordination::sealed_bid_auction(
                black_box(&bids),
                black_box(sangha::coordination::AuctionType::SecondPrice),
            )
        })
    });
}

fn bench_hatfield_contagion(c: &mut Criterion) {
    let states: Vec<sangha::contagion::EmotionalState> = (0..100)
        .map(|i| sangha::contagion::EmotionalState::new(i as f64 / 100.0, 0.8).unwrap())
        .collect();
    // Ring topology
    let adj: Vec<Vec<(usize, f64)>> = (0..100)
        .map(|i| vec![((i + 1) % 100, 1.0), ((i + 99) % 100, 1.0)])
        .collect();
    let config = sangha::contagion::HatfieldConfig::new(0.5, 0.0).unwrap();
    c.bench_function("contagion/hatfield_100", |b| {
        b.iter(|| {
            sangha::contagion::hatfield_contagion_step(
                black_box(&states),
                black_box(&adj),
                black_box(&config),
                black_box(0.01),
            )
        })
    });
}

fn bench_sis_step(c: &mut Criterion) {
    c.bench_function("contagion/sis_step", |b| {
        b.iter(|| {
            sangha::contagion::sis_step(
                black_box(0.9),
                black_box(0.1),
                black_box(0.5),
                black_box(0.2),
                black_box(0.01),
            )
        })
    });
}

fn bench_mood_propagation(c: &mut Criterion) {
    let moods: Vec<f64> = (0..100).map(|i| i as f64 / 100.0).collect();
    let adj: Vec<Vec<(usize, f64)>> = (0..100)
        .map(|i| vec![((i + 1) % 100, 1.0), ((i + 99) % 100, 1.0)])
        .collect();
    c.bench_function("contagion/mood_propagation_100", |b| {
        b.iter(|| {
            sangha::contagion::mood_propagation(
                black_box(&moods),
                black_box(&adj),
                black_box(0.1),
                black_box(0.01),
            )
        })
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
    bench_borda_count,
    bench_jury_theorem,
    bench_shapley_value,
    bench_is_core_stable,
    bench_public_goods_round,
    bench_sealed_bid_auction,
    bench_hatfield_contagion,
    bench_sis_step,
    bench_mood_propagation,
);
criterion_main!(benches);
