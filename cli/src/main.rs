use clap::{Parser, Subcommand, ValueEnum};
use encoding_rs::Encoding;
use engine::{Ability, AbilityScores, Actor, AdMode, Dice, Skill};
use serde::Deserialize;
use std::{collections::HashSet, fs, path::PathBuf};

#[derive(Copy, Clone, ValueEnum)]
enum Adv {
    Normal,
    Advantage,
    Disadvantage,
}

#[derive(Copy, Clone, ValueEnum)]
enum AbilityChoice {
    Auto,
    Str,
    Dex,
}

#[derive(Copy, Clone, ValueEnum)]
enum DType {
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

#[derive(Deserialize, Clone)]
struct TargetAttack {
    name: String,
    #[serde(rename = "to_hit")]
    to_hit: i32,
    dice: engine::DamageDice,
    #[serde(default)]
    damage_type: Option<engine::DamageType>,
}

#[derive(Deserialize, Clone)]
struct Target {
    name: String,
    ac: i32,
    hp: i32,
    #[serde(default)]
    dex_mod: i32,
    #[serde(default)]
    attacks: Vec<TargetAttack>,
    #[serde(default)]
    resistances: Vec<String>,
    #[serde(default)]
    vulnerabilities: Vec<String>,
    #[serde(default)]
    immunities: Vec<String>,
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
        /// Weapon preset (longsword, shortsword, dagger, greatsword, longbow)
        #[arg(long, default_value = "longsword")]
        weapon: String,
        /// Override damage dice (e.g., 1d8). If omitted, uses preset.
        #[arg(long)]
        dice: Option<String>,
        /// Override damage type (else from weapon/file or sensible preset)
        #[arg(long)]
        dtype: Option<DType>,
        /// Optional path to a weapons JSON file
        #[arg(long)]
        weapons: Option<PathBuf>,
        /// Ability selection: auto | str | dex
        #[arg(long, value_enum, default_value_t = AbilityChoice::Auto)]
        ability: AbilityChoice,
        /// Disable proficiency bonus
        #[arg(long, default_value_t = false)]
        no_prof: bool,
        /// Use versatile damage (two-handed) if available
        #[arg(long, default_value_t = false)]
        two_handed: bool,
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
    /// Attack a target loaded from JSON; supports one or multiple rounds
    AttackVs {
        /// Path to target JSON (name, ac, hp)
        #[arg(long)]
        target: PathBuf,

        /// Rounds to run (default 1). Stops early if target drops to 0 HP.
        #[arg(long, default_value_t = 1)]
        rounds: u32,

        /// Weapon preset (or use --dice to override)
        #[arg(long, default_value = "longsword")]
        weapon: String,

        /// Override damage dice (XdY). If omitted, uses weapon preset/file.
        #[arg(long)]
        dice: Option<String>,
        /// Override damage type (else from weapon/file or sensible preset)
        #[arg(long)]
        dtype: Option<DType>,

        /// Ability: auto | str | dex
        #[arg(long, value_enum, default_value_t = AbilityChoice::Auto)]
        ability: AbilityChoice,

        /// Disable proficiency bonus
        #[arg(long, default_value_t = false)]
        no_prof: bool,

        /// Optional weapons JSON file (falls back to content/weapons/basic.json then built-ins)
        #[arg(long)]
        weapons: Option<PathBuf>,

        /// Use versatile damage (two-handed) if available
        #[arg(long, default_value_t = false)]
        two_handed: bool,

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
    /// Full two-sided duel vs a target (initiative, alternating turns)
    Duel {
        /// Path to target JSON
        #[arg(long)]
        target: PathBuf,

        /// Actor AC (until we model armor/shield, pass it in)
        #[arg(long, default_value_t = 16)]
        actor_ac: i32,

        /// Actor HP (until we model hit dice/level, pass it in)
        #[arg(long, default_value_t = 12)]
        actor_hp: i32,

        /// Rounds safety cap (prevents infinite loops)
        #[arg(long, default_value_t = 20)]
        max_rounds: u32,

        /// Weapon preset (or override with --dice)
        #[arg(long, default_value = "longsword")]
        weapon: String,

        /// Override actor damage dice (XdY)
        #[arg(long)]
        dice: Option<String>,

        /// Ability: auto | str | dex
        #[arg(long, value_enum, default_value_t = AbilityChoice::Auto)]
        ability: AbilityChoice,

        /// Disable proficiency bonus for actor
        #[arg(long, default_value_t = false)]
        no_prof: bool,

        /// Use versatile damage (two-handed) if available
        #[arg(long, default_value_t = false)]
        two_handed: bool,

        /// Optional weapons JSON for presets
        #[arg(long)]
        weapons: Option<PathBuf>,

        /// Override actor damage type (else from weapon/preset)
        #[arg(long)]
        dtype: Option<DType>,

        /// RNG seed
        #[arg(long, default_value_t = 777)]
        seed: u64,

        /// Advantage mode for ACTOR only
        #[arg(long, value_enum, default_value_t = Adv::Normal)]
        adv: Adv,

        /// Optional actor JSON (else sample fighter)
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
                fs::write(path, s.as_bytes())?; // UTF-8, no BOM
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
            weapon,
            dice,
            dtype,
            weapons,
            ability,
            no_prof,
            two_handed,
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
            let resolved = resolve_weapon(&weapon, weapons.as_deref())?;
            let dtype = resolve_damage_type(dtype, &resolved);
            let chosen_ability = pick_ability(ability, &resolved);
            let proficient = !no_prof;

            // damage dice (override via --dice if provided)
            let dmg_spec = if let Some(ref s) = dice {
                parse_damage_dice(s)?
            } else if two_handed {
                resolved.versatile.unwrap_or(resolved.dice)
            } else {
                resolved.dice
            };

            let attack_bonus = actor.attack_bonus(chosen_ability, proficient);
            let damage_mod = actor.damage_mod(chosen_ability);

            let mut dice_rng = Dice::from_seed(seed);
            let mode = to_mode(adv);

            let atk = engine::attack(&mut dice_rng, mode, attack_bonus, ac);
            let is_crit = atk.nat20;
            let dmg = engine::damage(&mut dice_rng, dmg_spec, damage_mod, is_crit);

            let dmg_str = dice.clone().unwrap_or_else(|| dd_to_string(dmg_spec));

            println!(
                "attack: {} [{}] using {:?}: roll={} bonus={:+} total={} vs ac={} => {}{}",
                resolved.name,
                dmg_str,
                chosen_ability,
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
                "damage: {} + {:+}{} => {} [{:?}]",
                dmg_str,
                damage_mod,
                if is_crit { " (crit doubles dice)" } else { "" },
                dmg,
                dtype,
            );
        }
        Cmd::AttackVs {
            target,
            rounds,
            weapon,
            dice,
            dtype,
            ability,
            no_prof,
            weapons,
            two_handed,
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

            // Load target
            let mut tgt = read_target_auto(&target)?;
            let resist: HashSet<_> = tgt
                .resistances
                .iter()
                .filter_map(|s| parse_dtype_str(s))
                .collect();
            let vuln: HashSet<_> = tgt
                .vulnerabilities
                .iter()
                .filter_map(|s| parse_dtype_str(s))
                .collect();
            let immune: HashSet<_> = tgt
                .immunities
                .iter()
                .filter_map(|s| parse_dtype_str(s))
                .collect();

            let resolved = resolve_weapon(&weapon, weapons.as_deref())?;
            let dtype = resolve_damage_type(dtype, &resolved);
            let chosen_ability = pick_ability(ability, &resolved);
            let proficient = !no_prof;

            let dmg_spec = if let Some(ref s) = dice {
                parse_damage_dice(s)?
            } else if two_handed {
                resolved.versatile.unwrap_or(resolved.dice)
            } else {
                resolved.dice
            };
            let attack_bonus = actor.attack_bonus(chosen_ability, proficient);
            let damage_mod = actor.damage_mod(chosen_ability);

            let mut dice_rng = Dice::from_seed(seed);
            let mode = to_mode(adv);

            println!("target: {} (AC {}, HP {})", tgt.name, tgt.ac, tgt.hp);
            println!(
                "weapon: {} [{}] using {:?}{}",
                resolved.name,
                dd_to_string(dmg_spec),
                chosen_ability,
                if proficient {
                    " (proficient)"
                } else {
                    " (no prof)"
                }
            );

            for r in 1..=rounds {
                if tgt.hp <= 0 {
                    break;
                }
                let atk = engine::attack(&mut dice_rng, mode, attack_bonus, tgt.ac);
                let is_crit = atk.nat20;
                if atk.hit {
                    let raw = engine::damage(&mut dice_rng, dmg_spec, damage_mod, is_crit);
                    let dmg = engine::adjust_damage_by_type(raw, dtype, &resist, &vuln, &immune);
                    tgt.hp = (tgt.hp - dmg).max(0);
                    println!(
                        "round {}: HIT{} (roll={} total={}) dmg={} [{:?}] -> {} HP left",
                        r,
                        if atk.nat20 { " CRIT" } else { "" },
                        atk.roll,
                        atk.total,
                        dmg,
                        dtype,
                        tgt.hp
                    );
                } else {
                    println!(
                        "round {}: MISS{} (roll={} total={}) -> {} HP left",
                        r,
                        if atk.nat1 { " NAT1" } else { "" },
                        atk.roll,
                        atk.total,
                        tgt.hp
                    );
                }
            }
            if tgt.hp <= 0 {
                println!("{} is down.", tgt.name);
            }
        }
        Cmd::Duel {
            target,
            actor_ac,
            actor_hp,
            max_rounds,
            weapon,
            dice,
            ability,
            no_prof,
            two_handed,
            weapons,
            dtype,
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

            let tgt = read_target_auto(&target)?;
            let resist: HashSet<_> = tgt
                .resistances
                .iter()
                .filter_map(|s| parse_dtype_str(s))
                .collect();
            let vuln: HashSet<_> = tgt
                .vulnerabilities
                .iter()
                .filter_map(|s| parse_dtype_str(s))
                .collect();
            let immune: HashSet<_> = tgt
                .immunities
                .iter()
                .filter_map(|s| parse_dtype_str(s))
                .collect();

            let resolved = resolve_weapon(&weapon, weapons.as_deref())?;
            let actor_dtype = resolve_damage_type(dtype, &resolved);
            let chosen_ability = pick_ability(ability, &resolved);
            let proficient = !no_prof;

            let actor_dd = if let Some(ref s) = dice {
                parse_damage_dice(s)?
            } else if two_handed {
                resolved.versatile.unwrap_or(resolved.dice)
            } else {
                resolved.dice
            };

            let actor_atk_bonus = actor.attack_bonus(chosen_ability, proficient);
            let actor_dmg_mod = actor.damage_mod(chosen_ability);
            let actor_mode = to_mode(adv);

            let tgt_attack = tgt
                .attacks
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Target has no attacks"))?;
            let tgt_dtype = tgt_attack
                .damage_type
                .unwrap_or(engine::DamageType::Slashing);

            let mut rng = Dice::from_seed(seed);
            let actor_init = rng.d20(AdMode::Normal) as i32 + actor.ability_mod(Ability::Dex);
            let tgt_init = rng.d20(AdMode::Normal) as i32 + tgt.dex_mod;
            let mut actor_turn = actor_init >= tgt_init;

            let mut cur_actor_hp = actor_hp;
            let mut cur_tgt_hp = tgt.hp;

            println!(
                "Duel: Actor (AC {}, HP {}) vs {} (AC {}, HP {})",
                actor_ac, actor_hp, tgt.name, tgt.ac, tgt.hp
            );
            println!(
                "Initiative -> Actor {} vs {} {} => {} starts",
                actor_init,
                tgt.name,
                tgt_init,
                if actor_turn { "Actor" } else { &tgt.name }
            );
            println!(
                "Actor weapon: {} [{}] {:?}",
                resolved.name,
                dd_to_string(actor_dd),
                actor_dtype
            );
            println!("---");

            for round in 1..=max_rounds {
                if cur_actor_hp <= 0 || cur_tgt_hp <= 0 {
                    break;
                }
                println!("Round {}", round);

                if actor_turn {
                    let atk = engine::attack(&mut rng, actor_mode, actor_atk_bonus, tgt.ac);
                    if atk.hit {
                        let is_crit = atk.nat20;
                        let raw = engine::damage(&mut rng, actor_dd, actor_dmg_mod, is_crit);
                        let adj = engine::adjust_damage_by_type(
                            raw,
                            actor_dtype,
                            &resist,
                            &vuln,
                            &immune,
                        );
                        cur_tgt_hp = (cur_tgt_hp - adj).max(0);
                        println!(
                            "Actor HIT{} (roll={} total={}) dmg={} [{:?}] -> {} HP left",
                            if atk.nat20 { " CRIT" } else { "" },
                            atk.roll,
                            atk.total,
                            adj,
                            actor_dtype,
                            cur_tgt_hp
                        );
                    } else {
                        println!(
                            "Actor MISS{} (roll={} total={}) -> {} HP left",
                            if atk.nat1 { " NAT1" } else { "" },
                            atk.roll,
                            atk.total,
                            cur_tgt_hp
                        );
                    }
                } else {
                    let atk = engine::attack(&mut rng, AdMode::Normal, tgt_attack.to_hit, actor_ac);
                    if atk.hit {
                        let is_crit = atk.nat20;
                        let dmg = engine::damage(&mut rng, tgt_attack.dice, 0, is_crit);
                        cur_actor_hp = (cur_actor_hp - dmg).max(0);
                        println!(
                            "{} {} HIT{} (roll={} total={}) dmg={} [{:?}] -> Actor {} HP left",
                            tgt.name,
                            &tgt_attack.name,
                            if atk.nat20 { " CRIT" } else { "" },
                            atk.roll,
                            atk.total,
                            dmg,
                            tgt_dtype,
                            cur_actor_hp
                        );
                    } else {
                        println!(
                            "{} {} MISS{} (roll={} total={}) -> Actor {} HP left",
                            tgt.name,
                            &tgt_attack.name,
                            if atk.nat1 { " NAT1" } else { "" },
                            atk.roll,
                            atk.total,
                            cur_actor_hp
                        );
                    }
                }

                if cur_actor_hp <= 0 || cur_tgt_hp <= 0 {
                    break;
                }
                actor_turn = !actor_turn;
            }

            println!("---");
            if cur_tgt_hp <= 0 && cur_actor_hp > 0 {
                println!("Result: Actor defeats {}.", tgt.name);
            } else if cur_actor_hp <= 0 && cur_tgt_hp > 0 {
                println!("Result: {} defeats Actor.", tgt.name);
            } else if cur_actor_hp <= 0 && cur_tgt_hp <= 0 {
                println!("Result: Mutual KO.");
            } else {
                println!(
                    "Result: Max rounds reached ({} HP vs {} HP).",
                    cur_actor_hp, cur_tgt_hp
                );
            }
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

fn read_target_auto(path: &std::path::Path) -> anyhow::Result<Target> {
    let text = read_text_auto(path)?;
    Ok(serde_json::from_str(&text)?)
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

fn load_weapons_file(path: &std::path::Path) -> anyhow::Result<Vec<engine::Weapon>> {
    let text = read_text_auto(path)?;
    let v: Vec<engine::Weapon> = serde_json::from_str(&text)?;
    Ok(v)
}

fn find_weapon_in<'a>(name: &str, list: &'a [engine::Weapon]) -> Option<&'a engine::Weapon> {
    list.iter().find(|w| w.name.eq_ignore_ascii_case(name))
}

fn dd_to_string(dd: engine::DamageDice) -> String {
    format!("{}d{}", dd.count, dd.sides)
}

#[derive(Clone)]
struct ResolvedWeapon {
    name: String,
    dice: engine::DamageDice,
    finesse: bool,
    ranged: bool,
    versatile: Option<engine::DamageDice>,
    damage_type: Option<engine::DamageType>,
}

fn resolve_weapon(
    weapon: &str,
    weapons_path: Option<&std::path::Path>,
) -> anyhow::Result<ResolvedWeapon> {
    let loaded: Option<Vec<engine::Weapon>> = if let Some(path) = weapons_path {
        load_weapons_file(path).ok()
    } else {
        let default = std::path::Path::new("content/weapons/basic.json");
        load_weapons_file(default).ok()
    };

    if let Some(ref list) = loaded {
        if let Some(w) = find_weapon_in(weapon, list) {
            return Ok(ResolvedWeapon {
                name: w.name.clone(),
                dice: w.dice,
                finesse: w.finesse,
                ranged: w.ranged,
                versatile: w.versatile,
                damage_type: w.damage_type,
            });
        }
    }

    let preset = find_weapon(weapon).unwrap_or(WEAPONS[0]);
    Ok(ResolvedWeapon {
        name: preset.name.to_string(),
        dice: parse_damage_dice(preset.dice)?,
        finesse: preset.finesse,
        ranged: preset.ranged,
        versatile: match preset.versatile {
            Some(s) => Some(parse_damage_dice(s)?),
            None => None,
        },
        damage_type: preset_damage_type(preset.name),
    })
}

fn resolve_damage_type(dtype: Option<DType>, weapon: &ResolvedWeapon) -> engine::DamageType {
    dtype
        .map(to_engine_dtype)
        .or(weapon.damage_type)
        .unwrap_or(engine::DamageType::Slashing)
}

fn pick_ability(choice: AbilityChoice, weapon: &ResolvedWeapon) -> Ability {
    match choice {
        AbilityChoice::Str => Ability::Str,
        AbilityChoice::Dex => Ability::Dex,
        AbilityChoice::Auto => {
            if weapon.ranged || weapon.finesse {
                Ability::Dex
            } else {
                Ability::Str
            }
        }
    }
}

fn to_engine_dtype(dt: DType) -> engine::DamageType {
    use engine::DamageType as E;
    match dt {
        DType::Bludgeoning => E::Bludgeoning,
        DType::Piercing => E::Piercing,
        DType::Slashing => E::Slashing,
        DType::Fire => E::Fire,
        DType::Cold => E::Cold,
        DType::Lightning => E::Lightning,
        DType::Acid => E::Acid,
        DType::Poison => E::Poison,
        DType::Psychic => E::Psychic,
        DType::Radiant => E::Radiant,
        DType::Necrotic => E::Necrotic,
        DType::Thunder => E::Thunder,
        DType::Force => E::Force,
    }
}

fn parse_dtype_str(s: &str) -> Option<engine::DamageType> {
    use engine::DamageType::*;
    match &*s.to_lowercase() {
        "bludgeoning" => Some(Bludgeoning),
        "piercing" => Some(Piercing),
        "slashing" => Some(Slashing),
        "fire" => Some(Fire),
        "cold" => Some(Cold),
        "lightning" => Some(Lightning),
        "acid" => Some(Acid),
        "poison" => Some(Poison),
        "psychic" => Some(Psychic),
        "radiant" => Some(Radiant),
        "necrotic" => Some(Necrotic),
        "thunder" => Some(Thunder),
        "force" => Some(Force),
        _ => None,
    }
}

fn preset_damage_type(name: &str) -> Option<engine::DamageType> {
    match name.to_lowercase().as_str() {
        "longsword" | "greatsword" => Some(engine::DamageType::Slashing),
        "shortsword" | "dagger" | "longbow" => Some(engine::DamageType::Piercing),
        _ => None,
    }
}

/* ---------- weapon presets ---------- */

#[derive(Copy, Clone)]
struct WeaponPreset {
    name: &'static str,
    dice: &'static str, // "XdY"
    finesse: bool,
    ranged: bool,
    versatile: Option<&'static str>, // two-handed dice like "1d10"
}

const WEAPONS: &[WeaponPreset] = &[
    WeaponPreset {
        name: "longsword",
        dice: "1d8",
        finesse: false,
        ranged: false,
        versatile: Some("1d10"),
    },
    WeaponPreset {
        name: "shortsword",
        dice: "1d6",
        finesse: true,
        ranged: false,
        versatile: None,
    },
    WeaponPreset {
        name: "dagger",
        dice: "1d4",
        finesse: true,
        ranged: false,
        versatile: None,
    },
    WeaponPreset {
        name: "greatsword",
        dice: "2d6",
        finesse: false,
        ranged: false,
        versatile: None,
    },
    WeaponPreset {
        name: "longbow",
        dice: "1d8",
        finesse: false,
        ranged: true,
        versatile: None,
    },
];

fn find_weapon(name: &str) -> Option<WeaponPreset> {
    WEAPONS
        .iter()
        .copied()
        .find(|w| w.name.eq_ignore_ascii_case(name))
}
