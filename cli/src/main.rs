use clap::{Parser, Subcommand, ValueEnum};
use engine::{AdMode, Dice, Ability, AbilityScores, Actor, Skill};
use std::collections::HashSet;

#[derive(Copy, Clone, ValueEnum)]
enum Adv { Normal, Advantage, Disadvantage }

#[derive(Subcommand)]
enum Cmd {
    /// Roll a d20 multiple times with optional advantage/disadvantage
    Roll {
        /// RNG seed for determinism
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Advantage mode
        #[arg(long, value_enum, default_value_t = Adv::Normal)]
        adv: Adv,
        /// Number of rolls
        #[arg(long, default_value_t = 5)]
        rolls: u32,
    },
    /// Perform a check against a DC using a modifier and (dis)advantage
    Check {
        /// RNG seed for determinism
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Advantage mode
        #[arg(long, value_enum, default_value_t = Adv::Normal)]
        adv: Adv,
        /// Difficulty Class to beat (>=)
        #[arg(long)]
        dc: i32,
        /// Ability/skill modifier to add to the d20
        #[arg(long, default_value_t = 0)]
        modifier: i32,
    },
    /// Demo: run a few checks with a baked-in L1 Fighter
    ActorDemo {
        /// RNG seed for determinism
        #[arg(long, default_value_t = 222)]
        seed: u64,
        /// Advantage mode applied to all demo rolls
        #[arg(long, value_enum, default_value_t = Adv::Normal)]
        adv: Adv,
        /// DC to test against
        #[arg(long, default_value_t = 13)]
        dc: i32,
    },
}

#[derive(Parser)]
#[command(name = "solo5e-cli")]
#[command(about = "Solo5e CLI harness")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

fn to_mode(a: Adv) -> AdMode {
    match a {
        Adv::Normal => AdMode::Normal,
        Adv::Advantage => AdMode::Advantage,
        Adv::Disadvantage => AdMode::Disadvantage,
    }
}

fn sample_fighter() -> Actor {
    // L1 Fighter: PB +2, STR/CON saves; Athletics & Perception proficient
    let abilities = AbilityScores { str_: 16, dex: 14, con: 14, int_: 10, wis: 12, cha: 8 };
    let mut save = HashSet::new();
    save.insert(Ability::Str);
    save.insert(Ability::Con);
    let mut skills = HashSet::new();
    skills.insert(Skill::Athletics);
    skills.insert(Skill::Perception);
    Actor { abilities, proficiency_bonus: 2, save_proficiencies: save, skill_proficiencies: skills }
}

fn main() {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Roll { seed, adv, rolls } => {
            let mode = to_mode(adv);
            let mut dice = Dice::from_seed(seed);
            for _ in 0..rolls {
                println!("{}", dice.d20(mode));
            }
        }
        Cmd::Check { seed, adv, dc, modifier } => {
            let mode = to_mode(adv);
            let mut dice = Dice::from_seed(seed);
            let res = engine::check(&mut dice, engine::CheckInput { dc, modifier, mode });
            println!("roll={} mod={} total={} dc={} => {}", res.roll, modifier, res.total, res.dc, if res.passed { "SUCCESS" } else { "FAIL" });
        }
        Cmd::ActorDemo { seed, adv, dc } => {
            let mode = to_mode(adv);
            let actor = sample_fighter();
            let mut dice = Dice::from_seed(seed);

            // Ability check: STR
            let str_mod = actor.ability_mod(Ability::Str);
            let a = actor.ability_check(&mut dice, Ability::Str, mode, dc);
            println!("ability STR (mod={:+}): roll={} total={} vs dc={} => {}", str_mod, a.roll, a.total, a.dc, if a.passed { "SUCCESS" } else { "FAIL" });

            // Skill check: Athletics
            let ath_mod = actor.skill_mod(Skill::Athletics);
            let s = actor.skill_check(&mut dice, Skill::Athletics, mode, dc);
            println!("skill Athletics (mod={:+}): roll={} total={} vs dc={} => {}", ath_mod, s.roll, s.total, s.dc, if s.passed { "SUCCESS" } else { "FAIL" });

            // Saving throw: CON
            let con_mod = actor.save_mod(Ability::Con);
            let sv = actor.saving_throw(&mut dice, Ability::Con, mode, dc);
            println!("save CON (mod={:+}): roll={} total={} vs dc={} => {}", con_mod, sv.roll, sv.total, sv.dc, if sv.passed { "SUCCESS" } else { "FAIL" });
        }
    }
}
