use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AdMode {
    Normal,
    Advantage,
    Disadvantage,
}

pub struct Dice {
    rng: ChaCha8Rng,
}

impl Dice {
    pub fn from_seed(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }
    pub fn d20(&mut self, mode: AdMode) -> u8 {
        let mut roll = || self.rng.gen_range(1..=20);
        match mode {
            AdMode::Normal => roll(),
            AdMode::Advantage => {
                let a = roll();
                let b = roll();
                a.max(b)
            }
            AdMode::Disadvantage => {
                let a = roll();
                let b = roll();
                a.min(b)
            }
        }
    }
}
