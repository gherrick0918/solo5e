use clap::{Parser, Subcommand, ValueEnum};
use encoding_rs::Encoding;
use engine::{Ability, AbilityScores, Actor, AdMode, Dice, Skill};
use std::{collections::HashSet, fs, path::PathBuf};

#[derive(Copy, Clone, ValueEnum)]
enum Adv {
    Normal,
    Advantage,
    Disadvantage,
}

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
    /// Serialize the sample Fighter actor to JSON (stdout or file)
    ActorDump {
        /// Pretty-print JSON
        #[arg(long, default_value_t = true)]
        pretty: bool,
        /// Optional output path; if omitted, prints to stdout
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Load an Actor from a JSON file and run the demo checks
    ActorLoad {
        /// Path to JSON file containing an Actor
        #[arg(long)]
        file: PathBuf,
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
    /// Demo a basic weapon attack + damage
    AttackDemo {
        /// AC to hit
        #[arg(long, default_value_t = 13)]
        ac: i32,
        /// Damage dice, e.g. 1d8
        #[arg(long, default_value = "1d8")]
        dice: String,
        /// RNG seed
        #[arg(long, default_value_t = 123)]
        seed: u64,
        /// Advantage mode
        #[arg(long, value_enum, default_value_t = Adv::Normal)]
        adv: Adv,
        /// Optional actor JSON (if omitted, uses sample fighter)
        #[arg(long)]
        file: Option<PathBuf>,
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
    let abilities = AbilityScores {
        str_: 16,
        dex: 14,
        con: 14,
        int_: 10,
        wis: 12,
        cha: 8,
    };
    let mut save = HashSet::new();
    save.insert(Ability::Str);
    save.insert(Ability::Con);
    let mut skills = HashSet::new();
    skills.insert(Skill::Athletics);
    skills.insert(Skill::Perception);
    Actor {
        abilities,
        proficiency_bonus: 2,
        save_proficiencies: save,
        skill_proficiencies: skills,
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Roll { seed, adv, rolls } => {
            let mode = to_mode(adv);
            let mut dice = Dice::from_seed(seed);
            for _ in 0..rolls {
                println!("{}", dice.d20(mode));
            }
        }
        Cmd::Check {
            seed,
            adv,
            dc,
            modifier,
        } => {
            let mode = to_mode(adv);
            let mut dice = Dice::from_seed(seed);
            let res = engine::check(&mut dice, engine::CheckInput { dc, modifier, mode });
            println!(
                "roll={} mod={} total={} dc={} => {}",
                res.roll,
                modifier,
                res.total,
                res.dc,
                if res.passed { "SUCCESS" } else { "FAIL" }
            );
        }
        Cmd::ActorDemo { seed, adv, dc } => {
            let mode = to_mode(adv);
            let actor = sample_fighter();
            demo_checks(actor, seed, mode, dc);
        }
        Cmd::ActorDump { pretty, out } => {
            let actor = sample_fighter();
            let s = if pretty {
                serde_json::to_string_pretty(&actor)?
            } else {
                serde_json::to_string(&actor)?
            };
            if let Some(path) = out {
                fs::write(path, s.as_bytes())?;
            } else {
                println!("{}", s);
            }
        }
        Cmd::ActorLoad {
            file,
            seed,
            adv,
            dc,
        } => {
            let text = read_text_auto(&file)?;
            let actor: Actor = serde_json::from_str(&text)?;
            let mode = to_mode(adv);
            demo_checks(actor, seed, mode, dc);
        }
        Cmd::AttackDemo {
            ac,
            dice,
            seed,
            adv,
            file,
        } => {
            let actor = if let Some(path) = file {
                let text = read_text_auto(&path)?;
                serde_json::from_str::<Actor>(&text)?
            } else {
                sample_fighter()
            };

            // STR weapon, proficient
            let attack_bonus = actor.attack_bonus(Ability::Str, true);
            let damage_mod = actor.damage_mod(Ability::Str);

            let mut dice_rng = Dice::from_seed(seed);
            let mode = to_mode(adv);
            let dmg_spec = parse_damage_dice(&dice)?;

            let atk = engine::attack(&mut dice_rng, mode, attack_bonus, ac);
            let is_crit = atk.nat20;
            let dmg = engine::damage(&mut dice_rng, dmg_spec, damage_mod, is_crit);

            println!(
                "attack: roll={} bonus={:+} total={} vs ac={} => {}{}",
                atk.roll,
                atk.bonus,
                atk.total,
                atk.ac,
                if atk.hit { "HIT" } else { "MISS" },
                if atk.nat20 {
                    " (CRIT)"
                } else if atk.nat1 {
                    " (NAT1)"
                } else {
                    ""
                }
            );
            println!(
                "damage: {} + {:+}{} => {}",
                dice,
                damage_mod,
                if is_crit { " (crit doubles dice)" } else { "" },
                dmg
            );
        }
    }
    Ok(())
}

fn demo_checks(actor: Actor, seed: u64, mode: AdMode, dc: i32) {
    let mut dice = Dice::from_seed(seed);

    // Ability check: STR
    let str_mod = actor.ability_mod(Ability::Str);
    let a = actor.ability_check(&mut dice, Ability::Str, mode, dc);
    println!(
        "ability STR (mod={:+}): roll={} total={} vs dc={} => {}",
        str_mod,
        a.roll,
        a.total,
        a.dc,
        if a.passed { "SUCCESS" } else { "FAIL" }
    );

    // Skill check: Athletics
    let ath_mod = actor.skill_mod(Skill::Athletics);
    let s = actor.skill_check(&mut dice, Skill::Athletics, mode, dc);
    println!(
        "skill Athletics (mod={:+}): roll={} total={} vs dc={} => {}",
        ath_mod,
        s.roll,
        s.total,
        s.dc,
        if s.passed { "SUCCESS" } else { "FAIL" }
    );

    // Saving throw: CON
    let con_mod = actor.save_mod(Ability::Con);
    let sv = actor.saving_throw(&mut dice, Ability::Con, mode, dc);
    println!(
        "save CON (mod={:+}): roll={} total={} vs dc={} => {}",
        con_mod,
        sv.roll,
        sv.total,
        sv.dc,
        if sv.passed { "SUCCESS" } else { "FAIL" }
    );
}

fn read_text_auto(path: &std::path::Path) -> anyhow::Result<String> {
    let bytes = fs::read(path)?;
    if let Some((enc, bom_len)) = Encoding::for_bom(&bytes) {
        let (cow, _, _) = enc.decode(&bytes[bom_len..]);
        Ok(cow.into_owned())
    } else {
        Ok(String::from_utf8(bytes)?)
    }
}

fn parse_damage_dice(s: &str) -> anyhow::Result<engine::DamageDice> {
    let parts: Vec<_> = s.to_lowercase().split('d').collect();
    if parts.len() != 2 {
        anyhow::bail!("invalid dice spec (expected XdY), got: {}", s);
    }
    let count: u8 = parts[0].parse()?;
    let sides: u8 = parts[1].parse()?;
    if count == 0 || sides < 2 {
        anyhow::bail!("dice must be >= 1d2");
    }
    Ok(engine::DamageDice::new(count, sides))
}
