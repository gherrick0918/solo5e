use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdMode {
    Normal,
    Advantage,
    Disadvantage,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DamageType {
    Bludgeoning,
    Piercing,
    Slashing,
    Fire,
    Cold,
    Lightning,
    Acid,
    Poison,
    Psychic,
    Radiant,
    Necrotic,
    Thunder,
    Force,
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

    /// Roll a generic die: 1..=sides
    pub fn die(&mut self, sides: u8) -> u8 {
        self.rng.gen_range(1..=sides)
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
    CheckResult {
        roll,
        total,
        dc: input.dc,
        passed: total >= input.dc,
    }
}

/// D&D ability modifier = floor((score - 10) / 2) for integer scores.
pub fn ability_mod(score: i32) -> i32 {
    (score - 10).div_euclid(2)
}

/* ---------------- abilities, skills, actor ---------------- */

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ability {
    Str,
    Dex,
    Con,
    Int,
    Wis,
    Cha,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Skill {
    Athletics, // STR
    Acrobatics,
    SleightOfHand,
    Stealth, // DEX
    Arcana,
    History,
    Investigation,
    Nature,
    Religion, // INT
    AnimalHandling,
    Insight,
    Medicine,
    Perception,
    Survival, // WIS
    Deception,
    Intimidation,
    Performance,
    Persuasion, // CHA
}

impl Skill {
    pub fn key_ability(&self) -> Ability {
        use Ability::*;
        match self {
            Skill::Athletics => Str,
            Skill::Acrobatics | Skill::SleightOfHand | Skill::Stealth => Dex,
            Skill::Arcana
            | Skill::History
            | Skill::Investigation
            | Skill::Nature
            | Skill::Religion => Int,
            Skill::AnimalHandling
            | Skill::Insight
            | Skill::Medicine
            | Skill::Perception
            | Skill::Survival => Wis,
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
    pub fn mod_of(&self, a: Ability) -> i32 {
        ability_mod(self.get(a))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Actor {
    pub abilities: AbilityScores,
    pub proficiency_bonus: i32,
    pub save_proficiencies: HashSet<Ability>,
    pub skill_proficiencies: HashSet<Skill>,
}

impl Actor {
    pub fn ability_mod(&self, a: Ability) -> i32 {
        self.abilities.mod_of(a)
    }

    pub fn save_mod(&self, a: Ability) -> i32 {
        let base = self.ability_mod(a);
        let prof = if self.save_proficiencies.contains(&a) {
            self.proficiency_bonus
        } else {
            0
        };
        base + prof
    }

    pub fn skill_mod(&self, s: Skill) -> i32 {
        let base = self.ability_mod(s.key_ability());
        let prof = if self.skill_proficiencies.contains(&s) {
            self.proficiency_bonus
        } else {
            0
        };
        base + prof
    }

    pub fn ability_check(&self, dice: &mut Dice, a: Ability, mode: AdMode, dc: i32) -> CheckResult {
        check(
            dice,
            CheckInput {
                dc,
                modifier: self.ability_mod(a),
                mode,
            },
        )
    }
    pub fn skill_check(&self, dice: &mut Dice, s: Skill, mode: AdMode, dc: i32) -> CheckResult {
        check(
            dice,
            CheckInput {
                dc,
                modifier: self.skill_mod(s),
                mode,
            },
        )
    }
    pub fn saving_throw(&self, dice: &mut Dice, a: Ability, mode: AdMode, dc: i32) -> CheckResult {
        check(
            dice,
            CheckInput {
                dc,
                modifier: self.save_mod(a),
                mode,
            },
        )
    }

    /// Attack bonus = ability mod + proficiency (if proficient)
    pub fn attack_bonus(&self, ability: Ability, proficient: bool) -> i32 {
        let mut b = self.ability_mod(ability);
        if proficient {
            b += self.proficiency_bonus;
        }
        b
    }

    /// Damage modifier typically the ability mod (e.g., STR for melee)
    pub fn damage_mod(&self, ability: Ability) -> i32 {
        self.ability_mod(ability)
    }
}

/* ---------------- attacks & damage ---------------- */

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct DamageDice {
    pub count: u8,
    pub sides: u8,
}

impl DamageDice {
    pub fn new(count: u8, sides: u8) -> Self {
        Self { count, sides }
    }

    pub fn roll_total(&self, dice: &mut Dice, crit: bool) -> i32 {
        let n = if crit {
            self.count.saturating_mul(2)
        } else {
            self.count
        } as i32;
        let mut sum = 0;
        for _ in 0..n {
            sum += dice.die(self.sides) as i32;
        }
        sum
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AttackResult {
    pub roll: i32,
    pub total: i32,
    pub ac: i32,
    pub bonus: i32,
    pub nat20: bool,
    pub nat1: bool,
    pub hit: bool,
}

/// 5e: nat20 always hits, nat1 always misses; otherwise total >= AC.
pub fn attack(dice: &mut Dice, mode: AdMode, bonus: i32, ac: i32) -> AttackResult {
    let r = dice.d20(mode) as i32;
    let nat20 = r == 20;
    let nat1 = r == 1;
    let total = r + bonus;
    let hit = if nat20 {
        true
    } else if nat1 {
        false
    } else {
        total >= ac
    };
    AttackResult {
        roll: r,
        total,
        ac,
        bonus,
        nat20,
        nat1,
        hit,
    }
}

/// On crit, double dice (modifier once).
pub fn damage(dice: &mut Dice, dice_spec: DamageDice, modifier: i32, crit: bool) -> i32 {
    dice_spec.roll_total(dice, crit) + modifier
}

pub fn adjust_damage_by_type(
    base: i32,
    dtype: DamageType,
    resist: &HashSet<DamageType>,
    vuln: &HashSet<DamageType>,
    immune: &HashSet<DamageType>,
) -> i32 {
    if immune.contains(&dtype) {
        return 0;
    }
    let has_res = resist.contains(&dtype);
    let has_vuln = vuln.contains(&dtype);
    if has_res && has_vuln {
        return base;
    }
    if has_res {
        (base as f32 / 2.0).floor() as i32
    } else if has_vuln {
        base * 2
    } else {
        base
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Weapon {
    pub name: String,
    pub dice: DamageDice,
    #[serde(default)]
    pub finesse: bool,
    #[serde(default)]
    pub ranged: bool,
    /// Optional versatile dice (e.g., longsword 1d10)
    #[serde(default)]
    pub versatile: Option<DamageDice>,
    #[serde(default)]
    pub damage_type: Option<DamageType>,
}
