use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

pub struct FIRO<T> {
    data: Vec<T>,
    rng: StdRng,
}

impl<T> FIRO<T> {
    pub fn new(seed: u64) -> Self {
        let rng = StdRng::seed_from_u64(seed);
        FIRO { data: Vec::new(), rng }
    }

    pub fn enqueue(&mut self, item: T) {
        self.data.push(item);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.data.is_empty() {
            None
        } else {
            let len = self.data.len();
            let random_index = self.rng.gen_range(0..len);
            self.data.swap(random_index, len - 1);
            self.data.pop()
        }
    }
}
