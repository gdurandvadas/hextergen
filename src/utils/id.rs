use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

pub fn new(seed: u64, len: usize) -> String {
    let mut rng = Pcg64Mcg::seed_from_u64(seed);
    let alphabet: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
        .chars()
        .collect();
    let random_string: String = (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..alphabet.len());
            alphabet[idx]
        })
        .collect();

    random_string
}
