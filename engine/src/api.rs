use std::{collections::HashSet, fs};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::conditions::{
    maybe_apply_on_hit_condition, process_turn_boundary, vantage_from_conditions, ActiveCondition,
    AttackStyle, ConditionKind, TurnBoundary, Vantage,
};
use crate::life::{apply_damage, process_death_save_start_of_turn, Health, LifeState};
use crate::{Ability, AbilityScores, Actor, AdMode, Cover, DamageDice, DamageType, Dice, Weapon};

const DEFAULT_ACTOR_AC: i32 = 16;
const DEFAULT_ACTOR_HP: i32 = 12;
const MAX_ROUNDS: u32 = 30;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DuelConfig {
    pub target_path: String,
    pub weapon: String,
    pub weapons_path: String,
    #[serde(default)]
    pub actor_conditions: Vec<String>,
    #[serde(default)]
    pub enemy_conditions: Vec<String>,
    #[serde(default)]
    pub seed: u64,
    #[serde(default)]
    pub actor_hp: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DuelResult {
    pub winner: String,
    pub rounds: u32,
    pub actor_hp_end: i32,
    pub enemy_hp_end: i32,
    pub log: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct TargetAttack {
    name: String,
    #[serde(rename = "to_hit")]
    to_hit: i32,
    dice: DamageDice,
    #[serde(default)]
    damage_type: Option<DamageType>,
    #[serde(default)]
    ranged: bool,
    #[serde(default)]
    apply_condition: Option<crate::conditions::ConditionSpec>,
}

#[derive(Debug, Clone, Deserialize)]
struct TargetData {
    name: String,
    ac: i32,
    hp: i32,
    #[serde(default)]
    dex_mod: i32,
    #[serde(default)]
    abilities: Option<AbilityScores>,
    #[serde(default)]
    attacks: Vec<TargetAttack>,
    #[serde(default)]
    resistances: Vec<String>,
    #[serde(default)]
    vulnerabilities: Vec<String>,
    #[serde(default)]
    immunities: Vec<String>,
    #[serde(default)]
    conditions: Vec<ConditionKind>,
    #[serde(default)]
    cover: Cover,
}

impl TargetData {
    fn dexterity_mod(&self) -> i32 {
        if let Some(scores) = &self.abilities {
            scores.mod_of(Ability::Dex)
        } else {
            self.dex_mod
        }
    }

    fn ability_mod(&self, ability: Ability) -> i32 {
        if let Some(scores) = &self.abilities {
            scores.mod_of(ability)
        } else if ability == Ability::Dex {
            self.dex_mod
        } else {
            0
        }
    }
}

pub fn simulate_duel(cfg: DuelConfig) -> Result<DuelResult> {
    let mut logs = Vec::new();

    let target = load_target(&cfg.target_path)?;
    if target.attacks.is_empty() {
        bail!("target has no attacks");
    }
    let target_attack = target.attacks[0].clone();

    let weapons = load_weapons(&cfg.weapons_path)?;
    let weapon = find_weapon(&weapons, &cfg.weapon)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("weapon '{}' not found", cfg.weapon))?;

    let actor = sample_fighter();
    let actor_hp = cfg.actor_hp.unwrap_or(DEFAULT_ACTOR_HP);
    let actor_ac = DEFAULT_ACTOR_AC;
    let mut actor_health = Health::new(actor_hp);
    let mut enemy_hp = target.hp;

    let actor_weapon_dice = weapon.versatile.unwrap_or(weapon.dice);
    let actor_damage_type = weapon
        .damage_type
        .or_else(|| preset_damage_type(&weapon.name));
    let actor_style = if weapon.ranged {
        AttackStyle::Ranged
    } else {
        AttackStyle::Melee
    };
    let actor_ability = if weapon.ranged || weapon.finesse {
        Ability::Dex
    } else {
        Ability::Str
    };
    let actor_attack_bonus = actor.attack_bonus(actor_ability, true);
    let actor_damage_mod = actor.damage_mod(actor_ability);
    let actor_mode: Vantage = AdMode::Normal.into();

    let mut actor_conditions: Vec<ActiveCondition> = Vec::new();
    for cond in parse_condition_list(&cfg.actor_conditions) {
        logs.push(format!("[COND][Actor] starts with {:?}", cond.kind));
        actor_conditions.push(cond);
    }

    let mut enemy_conditions: Vec<ActiveCondition> = Vec::new();
    for cond in target.conditions.iter().cloned() {
        logs.push(format!("[COND][{}] starts with {:?}", target.name, cond));
        enemy_conditions.push(ActiveCondition {
            kind: cond,
            save_ends_each_turn: false,
            end_phase: None,
            end_save: None,
            pending_one_turn: false,
        });
    }
    for cond in parse_condition_list(&cfg.enemy_conditions) {
        logs.push(format!(
            "[COND][{}] starts with {:?}",
            target.name, cond.kind
        ));
        enemy_conditions.push(cond);
    }

    let mut rng = Dice::from_seed(cfg.seed);
    let actor_init = rng.d20(AdMode::Normal) as i32 + actor.ability_mod(Ability::Dex);
    let enemy_init = rng.d20(AdMode::Normal) as i32 + target.dexterity_mod();
    let mut actor_turn = actor_init >= enemy_init;

    logs.push(format!(
        "[START] Actor (AC {}, HP {}) vs {} (AC {}, HP {})",
        actor_ac, actor_hp, target.name, target.ac, target.hp
    ));
    logs.push(format!(
        "[INIT] Actor {} vs {} {} → {} starts",
        actor_init,
        target.name,
        enemy_init,
        if actor_turn {
            "Actor"
        } else {
            target.name.as_str()
        }
    ));

    let resist: HashSet<_> = target
        .resistances
        .iter()
        .filter_map(|s| parse_damage_type(s))
        .collect();
    let vuln: HashSet<_> = target
        .vulnerabilities
        .iter()
        .filter_map(|s| parse_damage_type(s))
        .collect();
    let immune: HashSet<_> = target
        .immunities
        .iter()
        .filter_map(|s| parse_damage_type(s))
        .collect();

    let mut rounds = 0u32;
    while rounds < MAX_ROUNDS && !matches!(actor_health.state, LifeState::Dead) && enemy_hp > 0 {
        rounds += 1;
        logs.push(format!(
            "[ROUND] {} → {}",
            rounds,
            if actor_turn {
                "Actor"
            } else {
                target.name.as_str()
            }
        ));

        if actor_turn {
            if let Some(outcome) = process_death_save_start_of_turn(
                "Actor",
                &mut actor_health,
                || rng.d20(AdMode::Normal) as i32,
                |msg| logs.push(msg),
            ) {
                logs.push(format!("[TURN][Actor] death save: {}", outcome));
            }

            process_turn_boundary(
                TurnBoundary::StartOfTurn,
                "Actor",
                &mut actor_conditions,
                |ability, _dc| {
                    let roll = rng.d20(AdMode::Normal) as i32;
                    let total = roll + actor.save_mod(ability);
                    (roll, total)
                },
                |msg| logs.push(msg),
            );

            match actor_health.state {
                LifeState::Dead => {
                    logs.push("[TURN][Actor] is dead; skipping".to_string());
                }
                LifeState::Unconscious { .. } => {
                    logs.push("[TURN][Actor] is unconscious; skipping actions".to_string());
                }
                LifeState::Conscious => {
                    let cond_vantage =
                        vantage_from_conditions(&actor_conditions, &enemy_conditions, actor_style);
                    let final_mode: AdMode = actor_mode.combine(cond_vantage).into();
                    let effective_enemy_ac = target.ac + target.cover.ac_bonus();
                    log_defense(&mut logs, &target.name, target.ac, target.cover);
                    let atk =
                        crate::attack(&mut rng, final_mode, actor_attack_bonus, effective_enemy_ac);
                    log_attack(&mut logs, "Actor", &atk);
                    if atk.hit {
                        let is_crit = atk.is_crit;
                        let raw =
                            crate::damage(&mut rng, actor_weapon_dice, actor_damage_mod, is_crit);
                        let dtype = actor_damage_type.unwrap_or(DamageType::Slashing);
                        let dmg = crate::adjust_damage_by_type(raw, dtype, &resist, &vuln, &immune);
                        let before = enemy_hp;
                        enemy_hp = (enemy_hp - dmg).max(0);
                        log_damage(
                            &mut logs,
                            "Actor",
                            actor_weapon_dice,
                            actor_damage_mod,
                            is_crit,
                            dmg,
                            Some(dtype),
                        );
                        logs.push(format!("[HP][{}] {} → {}", target.name, before, enemy_hp));
                    }
                }
            }

            process_turn_boundary(
                TurnBoundary::EndOfTurn,
                "Actor",
                &mut actor_conditions,
                |ability, _dc| {
                    let roll = rng.d20(AdMode::Normal) as i32;
                    let total = roll + actor.save_mod(ability);
                    (roll, total)
                },
                |msg| logs.push(msg),
            );
        } else {
            process_turn_boundary(
                TurnBoundary::StartOfTurn,
                &target.name,
                &mut enemy_conditions,
                |ability, _dc| {
                    let roll = rng.d20(AdMode::Normal) as i32;
                    let total = roll + target.ability_mod(ability);
                    (roll, total)
                },
                |msg| logs.push(msg),
            );

            if enemy_hp > 0 {
                let cond_vantage = vantage_from_conditions(
                    &enemy_conditions,
                    &actor_conditions,
                    if target_attack.ranged {
                        AttackStyle::Ranged
                    } else {
                        AttackStyle::Melee
                    },
                );
                let final_mode: AdMode = Vantage::Normal.combine(cond_vantage).into();
                let effective_actor_ac = actor_ac + Cover::None.ac_bonus();
                log_defense(&mut logs, "Actor", actor_ac, Cover::None);
                let atk = crate::attack(
                    &mut rng,
                    final_mode,
                    target_attack.to_hit,
                    effective_actor_ac,
                );
                log_attack(&mut logs, &target_attack.name, &atk);
                if atk.hit {
                    let is_crit = atk.is_crit;
                    let dtype = target_attack.damage_type.unwrap_or(DamageType::Slashing);
                    let dmg = crate::damage(&mut rng, target_attack.dice, 0, is_crit);
                    log_damage(
                        &mut logs,
                        &target_attack.name,
                        target_attack.dice,
                        0,
                        is_crit,
                        dmg,
                        Some(dtype),
                    );
                    let dropped = apply_damage(
                        "Actor",
                        &mut actor_health,
                        &mut actor_conditions,
                        dmg,
                        |msg| logs.push(msg),
                    );
                    logs.push(format!("[HP][Actor] {} HP", actor_health.hp));
                    if dropped {
                        logs.push("[ITEM][Actor] drops to 0 HP".to_string());
                    }
                    if let Some(spec) = target_attack.apply_condition.as_ref() {
                        maybe_apply_on_hit_condition(
                            "Actor",
                            &mut actor_conditions,
                            spec,
                            |ability, _dc| {
                                let roll = rng.d20(AdMode::Normal) as i32;
                                let total = roll + actor.save_mod(ability);
                                (roll, total)
                            },
                            |msg| logs.push(msg),
                        );
                    }
                }
            }

            process_turn_boundary(
                TurnBoundary::EndOfTurn,
                &target.name,
                &mut enemy_conditions,
                |ability, _dc| {
                    let roll = rng.d20(AdMode::Normal) as i32;
                    let total = roll + target.ability_mod(ability);
                    (roll, total)
                },
                |msg| logs.push(msg),
            );
        }

        if matches!(actor_health.state, LifeState::Dead) || enemy_hp <= 0 {
            break;
        }
        actor_turn = !actor_turn;
    }

    let winner = if enemy_hp <= 0 && actor_health.hp > 0 {
        "actor"
    } else if enemy_hp <= 0 && actor_health.hp <= 0 {
        "draw"
    } else if matches!(actor_health.state, LifeState::Dead) || actor_health.hp <= 0 {
        "enemy"
    } else {
        "draw"
    };

    logs.push(format!(
        "[END] winner={} actor_hp={} enemy_hp={} rounds={}",
        winner, actor_health.hp, enemy_hp, rounds
    ));

    Ok(DuelResult {
        winner: winner.to_string(),
        rounds,
        actor_hp_end: actor_health.hp,
        enemy_hp_end: enemy_hp,
        log: logs,
    })
}

fn load_target(path: &str) -> Result<TargetData> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read target JSON: {}", path))?;
    let data = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse target JSON: {}", path))?;
    Ok(data)
}

fn load_weapons(path: &str) -> Result<Vec<Weapon>> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read weapons JSON: {}", path))?;
    let data = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse weapons JSON: {}", path))?;
    Ok(data)
}

fn find_weapon<'a>(weapons: &'a [Weapon], name: &str) -> Option<&'a Weapon> {
    weapons.iter().find(|w| w.name.eq_ignore_ascii_case(name))
}

fn parse_condition_list(src: &[String]) -> Vec<ActiveCondition> {
    src.iter()
        .filter_map(|s| match s.trim().to_lowercase().as_str() {
            "poisoned" => Some(ConditionKind::Poisoned),
            "prone" => Some(ConditionKind::Prone),
            "restrained" => Some(ConditionKind::Restrained),
            _ => None,
        })
        .map(|kind| ActiveCondition {
            kind,
            save_ends_each_turn: false,
            end_phase: None,
            end_save: None,
            pending_one_turn: false,
        })
        .collect()
}

fn parse_damage_type(s: &str) -> Option<DamageType> {
    use DamageType::*;
    match s.to_lowercase().as_str() {
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

fn preset_damage_type(name: &str) -> Option<DamageType> {
    match name.to_lowercase().as_str() {
        "longsword" | "greatsword" => Some(DamageType::Slashing),
        "shortsword" | "dagger" | "longbow" => Some(DamageType::Piercing),
        _ => None,
    }
}

fn sample_fighter() -> Actor {
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
    skills.insert(crate::Skill::Athletics);
    skills.insert(crate::Skill::Perception);
    Actor {
        abilities,
        proficiency_bonus: 2,
        save_proficiencies: save,
        skill_proficiencies: skills,
    }
}

fn format_d20_sequence(raw: &[u8], kept: i32) -> String {
    match raw {
        [] => format!("d20=? (keep={})", kept),
        [only] => format!("d20={} (keep={})", only, kept),
        [first, second] => format!("d20={} vs d20={} (keep={})", first, second, kept),
        _ => {
            let joined = raw
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!("d20s=[{}] (keep={})", joined, kept)
        }
    }
}

fn format_modifier(modifier: i32) -> String {
    if modifier >= 0 {
        format!("+{}", modifier)
    } else {
        format!("-{}", modifier.abs())
    }
}

fn log_attack(logs: &mut Vec<String>, name: &str, atk: &crate::AttackResult) {
    let rolls = format_d20_sequence(&atk.raw_rolls, atk.roll);
    let outcome = if atk.is_crit {
        "CRIT!"
    } else if atk.hit {
        "HIT"
    } else if atk.nat1 {
        "MISS (NAT1)"
    } else {
        "MISS"
    };
    let mark = if atk.hit { "✔" } else { "✖" };
    logs.push(format!(
        "[ATTACK][{}] {} → {} to-hit={} vs AC={} {}",
        name, rolls, outcome, atk.total, atk.ac, mark
    ));
}

fn log_damage(
    logs: &mut Vec<String>,
    name: &str,
    dice: DamageDice,
    modifier: i32,
    crit: bool,
    total: i32,
    dtype: Option<DamageType>,
) {
    let dice_expr = if crit {
        format!("2×({}d{})", dice.count, dice.sides)
    } else {
        format!("{}d{}", dice.count, dice.sides)
    };
    let prefix = if crit { "crit: " } else { "" };
    match dtype {
        Some(dt) => logs.push(format!(
            "[DMG][{}] {}rolled {} {} = {} [{:?}]",
            name,
            prefix,
            dice_expr,
            format_modifier(modifier),
            total,
            dt
        )),
        None => logs.push(format!(
            "[DMG][{}] {}rolled {} {} = {}",
            name,
            prefix,
            dice_expr,
            format_modifier(modifier),
            total
        )),
    }
}

fn log_defense(logs: &mut Vec<String>, name: &str, base_ac: i32, cover: Cover) {
    let bonus = cover.ac_bonus();
    logs.push(format!(
        "[DEF][{}] AC {} + cover({:+}) = {}",
        name,
        base_ac,
        bonus,
        base_ac + bonus
    ));
}
