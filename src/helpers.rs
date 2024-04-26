use rand::Rng;

pub fn random_data() -> Vec<[f64; 2]> {
    let mut rng = rand::thread_rng();
    (1..=1000)
        .map(|_| {
            let a: f64 = rng.gen_range(-400_000.0..=100_000_000.0);
            let b: f64 = rng.gen_range(-400_000.0..=100_000_000.0);
            [a, b]
        })
        .collect::<Vec<_>>()
}
