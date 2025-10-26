use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AdMode { Normal, Advantage, Disadvantage }

pub struct Dice { rng: ChaCha8Rng }

impl Dice {
    pub fn from_seed(seed: u64) -> Self {
        Self { rng: ChaCha8Rng::seed_from_u64(seed) }
    }

    pub fn d20(&mut self, mode: AdMode) -> u8 {
        let mut roll = || self.rng.gen_range(1..=20);
        match mode {
            AdMode::Normal => roll(),
            AdMode::Advantage => { let a = roll(); let b = roll(); a.max(b) }
            AdMode::Disadvantage => { let a = roll(); let b = roll(); a.min(b) }
        }
    }
}

/* ---------------- new: typed check API ---------------- */

#[derive(Debug, Clone, Copy)]
pub struct CheckInput {
    pub dc: i32,
    pub modifier: i32,
    pub mode: AdMode,
}

#[derive(Debug, Clone, Copy)]
pub struct CheckResult {
    pub roll: i32,
    pub total: i32,
    pub dc: i32,
    pub passed: bool,
}

/// Roll a d20 (with advantage/disadvantage), add modifier, compare vs DC.
pub fn check(dice: &mut Dice, input: CheckInput) -> CheckResult {
    let roll = dice.d20(input.mode) as i32;
    let total = roll + input.modifier;
    CheckResult { roll, total, dc: input.dc, passed: total >= input.dc }
}

/// D&D ability modifier = floor((score - 10) / 2) for integer scores.
pub fn ability_mod(score: i32) -> i32 {
    // `div_euclid` with positive divisor matches mathematical floor division.
    (score - 10).div_euclid(2)
}
