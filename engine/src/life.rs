use serde::{Deserialize, Serialize};

use crate::conditions::{ActiveCondition, ConditionKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifeState {
    Conscious,
    Unconscious { stable: bool },
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DeathSaves {
    pub successes: u8, // 0..=3
    pub failures: u8,  // 0..=3
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Health {
    pub hp: i32,
    pub max_hp: i32,
    pub state: LifeState,
    pub death: DeathSaves,
}

impl Health {
    pub fn new(max_hp: i32) -> Self {
        Self {
            hp: max_hp,
            max_hp,
            state: LifeState::Conscious,
            death: DeathSaves::default(),
        }
    }
}

/// Apply damage and handle drop-to-0 transitions. Returns true if the creature dropped to 0 this call.
pub fn apply_damage(
    name: &str,
    health: &mut Health,
    conditions: &mut Vec<ActiveCondition>,
    dmg: i32,
    mut log: impl FnMut(String),
) -> bool {
    if matches!(health.state, LifeState::Dead) {
        return false;
    }

    let before = health.hp;
    health.hp = (health.hp - dmg).max(0);
    log(format!(
        "[DMG][{}] {} → {} (−{})",
        name, before, health.hp, dmg
    ));

    if before > 0 && health.hp == 0 {
        // Transition to Unconscious (not stable). Apply Prone once for flavor.
        health.state = LifeState::Unconscious { stable: false };
        if !conditions.iter().any(|c| c.kind == ConditionKind::Prone) {
            conditions.push(ActiveCondition {
                kind: ConditionKind::Prone,
                save_ends_each_turn: false,
                end_phase: None,
                end_save: None,
                pending_one_turn: false,
            });
            log(format!("[COND][{}] gains Prone (unconscious)", name));
        }
        log(format!("[STATE][{}] drops to 0 HP → Unconscious", name));
        return true;
    }
    false
}

/// Healing; if at 0/unconscious, wakes and resets death saves.
pub fn heal(name: &str, health: &mut Health, amount: i32, mut log: impl FnMut(String)) {
    if amount <= 0 {
        return;
    }
    let before = health.hp;
    let was_uncon = matches!(health.state, LifeState::Unconscious { .. });
    health.hp = (health.hp + amount).min(health.max_hp);
    if was_uncon && health.hp > 0 {
        health.state = LifeState::Conscious;
        health.death = DeathSaves::default();
        log(format!(
            "[HEAL][{}] +{} HP ({} → {}) and regains consciousness",
            name, amount, before, health.hp
        ));
    } else {
        log(format!(
            "[HEAL][{}] +{} HP ({} → {})",
            name, amount, before, health.hp
        ));
    }
}

/// Stabilize an unconscious creature at 0 HP (no more death saves).
pub fn stabilize(name: &str, health: &mut Health, mut log: impl FnMut(String)) {
    if let LifeState::Unconscious { stable: _ } = health.state {
        health.state = LifeState::Unconscious { stable: true };
        log(format!("[STATE][{}] is stabilized at 0 HP", name));
    }
}

/// Call at the start of the creature’s turn (before actions).
/// Returns `Some(outcome_string)` when a roll happened (for logging by caller), else None.
pub fn process_death_save_start_of_turn(
    name: &str,
    health: &mut Health,
    mut d20: impl FnMut() -> i32,
    mut log: impl FnMut(String),
) -> Option<String> {
    match health.state {
        LifeState::Unconscious { stable } if !stable && health.hp == 0 => {
            let roll = d20();
            // Nat 20 → 1 HP and wake; Nat 1 → 2 fails; otherwise success/failure by 10+
            let note = if roll == 20 {
                health.death = DeathSaves::default();
                health.hp = 1;
                health.state = LifeState::Conscious;
                "NAT20 → regain 1 HP & wake".to_string()
            } else if roll == 1 {
                health.death.failures = (health.death.failures + 2).min(3);
                "NAT1 → 2 failures".to_string()
            } else if roll >= 10 {
                health.death.successes = (health.death.successes + 1).min(3);
                "success".to_string()
            } else {
                health.death.failures = (health.death.failures + 1).min(3);
                "failure".to_string()
            };

            // Resolve thresholds
            if health.death.failures >= 3 {
                health.state = LifeState::Dead;
                log(format!(
                    "[DEATHSAVE][{}] roll={} → failure tally={}, success tally={} → DEAD",
                    name, roll, health.death.failures, health.death.successes
                ));
                return Some(format!("roll={} → DEAD", roll));
            }
            if health.death.successes >= 3 {
                // Stabilized at 0 (still unconscious)
                health.state = LifeState::Unconscious { stable: true };
                log(format!(
                    "[DEATHSAVE][{}] roll={} → stabilized (3 successes)",
                    name, roll
                ));
                return Some(format!("roll={} → stabilized", roll));
            }

            log(format!(
                "[DEATHSAVE][{}] roll={} → {} (S={}, F={})",
                name, roll, note, health.death.successes, health.death.failures
            ));
            Some(format!("roll={} → {}", roll, note))
        }
        _ => None,
    }
}
