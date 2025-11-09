use clap::Parser;
use encoding_rs::Encoding;
use engine::{Ability, AbilityScores, Actor, AdMode, Dice, Skill};
use serde::Deserialize;
use std::{collections::HashSet, fs, path::PathBuf};

#[derive(Parser)]
#[command(name = "simulate-vs")]
#[command(about = "Monte Carlo sim: many fights vs a JSON target")]
struct Args {
    /// Path to target JSON (name, ac, hp)
    #[arg(long)]
    target: PathBuf,

    /// Number of trials
    #[arg(long, default_value_t = 1000)]
    trials: u32,

    /// Safety cap on rounds per trial
    #[arg(long, default_value_t = 20)]
    max_rounds: u32,

    /// Weapon preset (or override with --dice)
    #[arg(long, default_value = "longsword")]
    weapon: String,

    /// Override damage dice (XdY). If omitted, uses weapon preset/file.
    #[arg(long)]
    dice: Option<String>,

    /// Ability: auto | str | dex
    #[arg(long, default_value = "auto")]
    ability: String,

    /// Disable proficiency bonus
    #[arg(long, default_value_t = false)]
    no_prof: bool,

    /// Optional weapons JSON file (falls back to content/weapons/basic.json then built-ins)
    #[arg(long)]
    weapons: Option<PathBuf>,

    /// RNG base seed (trial i uses seed+i)
    #[arg(long, default_value_t = 12345)]
    seed: u64,

    /// Advantage mode: normal | advantage | disadvantage
    #[arg(long, default_value = "normal")]
    adv: String,

    /// Optional actor JSON (if omitted, uses sample fighter)
    #[arg(long)]
    file: Option<PathBuf>,
}

#[derive(Deserialize, Clone)]
struct Target {
    name: String,
    ac: i32,
    hp: i32,
}

#[derive(Copy, Clone)]
struct WeaponPreset {
    name: &'static str,
    dice: &'static str, // "XdY"
    finesse: bool,
    ranged: bool,
}

const WEAPONS: &[WeaponPreset] = &[
    WeaponPreset {
        name: "longsword",
        dice: "1d8",
        finesse: false,
        ranged: false,
    },
    WeaponPreset {
        name: "shortsword",
        dice: "1d6",
        finesse: true,
        ranged: false,
    },
    WeaponPreset {
        name: "dagger",
        dice: "1d4",
        finesse: true,
        ranged: false,
    },
    WeaponPreset {
        name: "greatsword",
        dice: "2d6",
        finesse: false,
        ranged: false,
    },
    WeaponPreset {
        name: "longbow",
        dice: "1d8",
        finesse: false,
        ranged: true,
    },
];

fn find_weapon(name: &str) -> Option<WeaponPreset> {
    WEAPONS
        .iter()
        .copied()
        .find(|w| w.name.eq_ignore_ascii_case(name))
}

fn to_mode(s: &str) -> AdMode {
    match s.to_lowercase().as_str() {
        "advantage" => AdMode::Advantage,
        "disadvantage" => AdMode::Disadvantage,
        _ => AdMode::Normal,
    }
}

fn parse_damage_dice(s: &str) -> anyhow::Result<engine::DamageDice> {
    let lowered = s.to_lowercase();
    let parts: Vec<_> = lowered.split('d').collect();
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

fn dd_to_string(dd: engine::DamageDice) -> String {
    format!("{}d{}", dd.count, dd.sides)
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

fn read_target_auto(path: &std::path::Path) -> anyhow::Result<Target> {
    let text = read_text_auto(path)?;
    Ok(serde_json::from_str(&text)?)
}

fn load_weapons_file(path: &std::path::Path) -> anyhow::Result<Vec<engine::Weapon>> {
    let text = read_text_auto(path)?;
    let v: Vec<engine::Weapon> = serde_json::from_str(&text)?;
    Ok(v)
}

fn find_weapon_in<'a>(name: &str, list: &'a [engine::Weapon]) -> Option<&'a engine::Weapon> {
    list.iter().find(|w| w.name.eq_ignore_ascii_case(name))
}

fn sample_fighter() -> Actor {
    // same as main.rs sample
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

fn pick_ability(choice: &str, finesse: bool, ranged: bool) -> Ability {
    match choice.to_lowercase().as_str() {
        "str" => Ability::Str,
        "dex" => Ability::Dex,
        _ => {
            if ranged || finesse {
                Ability::Dex
            } else {
                Ability::Str
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Actor
    let actor = if let Some(path) = args.file.as_ref() {
        let text = read_text_auto(path)?;
        serde_json::from_str::<Actor>(&text)?
    } else {
        sample_fighter()
    };

    // Target
    let base_tgt = read_target_auto(&args.target)?;

    // Weapons file (optional) or default, else built-ins
    let loaded: Option<Vec<engine::Weapon>> = if let Some(ref p) = args.weapons {
        load_weapons_file(p).ok()
    } else {
        let default = std::path::Path::new("content/weapons/basic.json");
        load_weapons_file(default).ok()
    };

    // Resolve weapon
    struct ResolvedWeapon {
        name: String,
        dice: engine::DamageDice,
        finesse: bool,
        ranged: bool,
    }
    let resolved = if let Some(ref list) = loaded {
        if let Some(w) = find_weapon_in(&args.weapon, list) {
            ResolvedWeapon {
                name: w.name.clone(),
                dice: w.dice,
                finesse: w.finesse,
                ranged: w.ranged,
            }
        } else {
            let p = find_weapon(&args.weapon).unwrap_or(WEAPONS[0]);
            ResolvedWeapon {
                name: p.name.to_string(),
                dice: parse_damage_dice(p.dice)?,
                finesse: p.finesse,
                ranged: p.ranged,
            }
        }
    } else {
        let p = find_weapon(&args.weapon).unwrap_or(WEAPONS[0]);
        ResolvedWeapon {
            name: p.name.to_string(),
            dice: parse_damage_dice(p.dice)?,
            finesse: p.finesse,
            ranged: p.ranged,
        }
    };

    // Ability & proficiency
    let chosen_ability = pick_ability(&args.ability, resolved.finesse, resolved.ranged);
    let proficient = !args.no_prof;

    // Damage dice selection
    let base_spec = if let Some(ref s) = args.dice {
        parse_damage_dice(s)?
    } else {
        resolved.dice
    };

    // Precompute
    let attack_bonus = actor.attack_bonus(chosen_ability, proficient);
    let damage_mod = actor.damage_mod(chosen_ability);
    let mode = to_mode(&args.adv);

    // Stats
    let mut wins = 0u32;
    let mut hit_count = 0u32;
    let mut crit_count = 0u32;
    let mut miss_count = 0u32;
    let mut dmg_total_on_hits = 0i64;
    let mut rounds_vec: Vec<u32> = Vec::with_capacity(args.trials as usize);

    for i in 0..args.trials {
        let mut tgt_hp = base_tgt.hp;
        let mut rounds = 0u32;
        let trial_seed = args.seed.wrapping_add(i as u64);
        let mut rng = Dice::from_seed(trial_seed);

        while rounds < args.max_rounds && tgt_hp > 0 {
            rounds += 1;
            let atk = engine::attack(&mut rng, mode, attack_bonus, base_tgt.ac);
            if atk.hit {
                let is_crit = atk.nat20;
                let dmg = engine::damage(&mut rng, base_spec, damage_mod, is_crit);
                if is_crit {
                    crit_count += 1;
                }
                hit_count += 1;
                dmg_total_on_hits += dmg as i64;
                tgt_hp = (tgt_hp - dmg).max(0);
            } else {
                miss_count += 1;
            }
        }

        if tgt_hp <= 0 {
            wins += 1;
            rounds_vec.push(rounds);
        }
    }

    rounds_vec.sort_unstable();
    let trials_f = args.trials as f64;
    let win_rate = wins as f64 / trials_f;
    let hit_rate = if hit_count + miss_count == 0 {
        0.0
    } else {
        hit_count as f64 / (hit_count + miss_count) as f64
    };
    let crit_rate = if hit_count == 0 {
        0.0
    } else {
        crit_count as f64 / hit_count as f64
    };
    let avg_dmg_per_hit = if hit_count == 0 {
        0.0
    } else {
        dmg_total_on_hits as f64 / hit_count as f64
    };
    let avg_rounds = if rounds_vec.is_empty() {
        0.0
    } else {
        (rounds_vec.iter().map(|&r| r as u64).sum::<u64>() as f64) / (wins.max(1)) as f64
    };
    let median_rounds = if rounds_vec.is_empty() {
        0
    } else {
        let m = rounds_vec.len() / 2;
        if rounds_vec.len() % 2 == 1 {
            rounds_vec[m]
        } else {
            (rounds_vec[m - 1] + rounds_vec[m]) / 2
        }
    };

    println!("simulate-vs results");
    println!("-------------------");
    println!("trials:             {}", args.trials);
    println!(
        "target:             {} (AC {}, HP {})",
        base_tgt.name, base_tgt.ac, base_tgt.hp
    );
    println!(
        "weapon:             {} [{}]",
        resolved.name,
        dd_to_string(base_spec)
    );
    println!("advantage:          {}", args.adv);
    println!("proficient:         {}", proficient);
    println!();
    println!("win rate:           {:.1}%", win_rate * 100.0);
    println!("hit rate:           {:.1}%", hit_rate * 100.0);
    println!("crit rate:          {:.1}%", crit_rate * 100.0);
    println!("avg dmg per hit:    {:.2}", avg_dmg_per_hit);
    println!("avg rounds (wins):  {:.2}", avg_rounds);
    println!("median rounds:      {}", median_rounds);

    Ok(())
}
