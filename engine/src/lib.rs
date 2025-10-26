use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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

/* ---------------- typed check API ---------------- */

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
    (score - 10).div_euclid(2)
}

/* ---------------- abilities, skills, actor ---------------- */

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ability { Str, Dex, Con, Int, Wis, Cha }

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Skill {
    Athletics,                // STR
    Acrobatics, SleightOfHand, Stealth, // DEX
    Arcana, History, Investigation, Nature, Religion, // INT
    AnimalHandling, Insight, Medicine, Perception, Survival, // WIS
    Deception, Intimidation, Performance, Persuasion, // CHA
}

impl Skill {
    pub fn key_ability(&self) -> Ability {
        use Ability::*;
        match self {
            Skill::Athletics => Str,
            Skill::Acrobatics | Skill::SleightOfHand | Skill::Stealth => Dex,
            Skill::Arcana | Skill::History | Skill::Investigation | Skill::Nature | Skill::Religion => Int,
            Skill::AnimalHandling | Skill::Insight | Skill::Medicine | Skill::Perception | Skill::Survival => Wis,
            Skill::Deception | Skill::Intimidation | Skill::Performance | Skill::Persuasion => Cha,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AbilityScores {
    #[serde(rename = "str")]
    pub str_: i32,
    #[serde(rename = "dex")]
    pub dex: i32,
    #[serde(rename = "con")]
    pub con: i32,
    #[serde(rename = "int")]
    pub int_: i32,
    #[serde(rename = "wis")]
    pub wis: i32,
    #[serde(rename = "cha")]
    pub cha: i32,
}

impl AbilityScores {
    pub fn get(&self, a: Ability) -> i32 {
        match a {
            Ability::Str => self.str_,
            Ability::Dex => self.dex,
            Ability::Con => self.con,
            Ability::Int => self.int_,
            Ability::Wis => self.wis,
            Ability::Cha => self.cha,
        }
    }
    pub fn mod_of(&self, a: Ability) -> i32 { ability_mod(self.get(a)) }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Actor {
    pub abilities: AbilityScores,
    pub proficiency_bonus: i32,
    pub save_proficiencies: HashSet<Ability>,
    pub skill_proficiencies: HashSet<Skill>,
}

impl Actor {
    pub fn ability_mod(&self, a: Ability) -> i32 { self.abilities.mod_of(a) }

    pub fn save_mod(&self, a: Ability) -> i32 {
        let base = self.ability_mod(a);
        let prof = if self.save_proficiencies.contains(&a) { self.proficiency_bonus } else { 0 };
        base + prof
    }

    pub fn skill_mod(&self, s: Skill) -> i32 {
        let base = self.ability_mod(s.key_ability());
        let prof = if self.skill_proficiencies.contains(&s) { self.proficiency_bonus } else { 0 };
        base + prof
    }

    pub fn ability_check(&self, dice: &mut Dice, a: Ability, mode: AdMode, dc: i32) -> CheckResult {
        check(dice, CheckInput { dc, modifier: self.ability_mod(a), mode })
    }
    pub fn skill_check(&self, dice: &mut Dice, s: Skill, mode: AdMode, dc: i32) -> CheckResult {
        check(dice, CheckInput { dc, modifier: self.skill_mod(s), mode })
    }
    pub fn saving_throw(&self, dice: &mut Dice, a: Ability, mode: AdMode, dc: i32) -> CheckResult {
        check(dice, CheckInput { dc, modifier: self.save_mod(a), mode })
    }
}
