use crate::{Ability, SavingThrow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionKind {
    Poisoned,
    Prone,
    Restrained,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DurationPhase {
    StartOfTurn,
    EndOfTurn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConditionDuration {
    /// If Some, the condition expires automatically on the affected creature's next occurrence of this phase.
    pub until: Option<DurationPhase>,
    /// If true, the affected creature attempts a saving throw at the end of each of its turns to end the condition.
    #[serde(default)]
    pub save_ends_each_turn: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionSpec {
    pub kind: ConditionKind,
    /// On application: if present, target makes this save to resist application.
    pub save: Option<SavingThrow>,
    /// How the condition lasts.
    #[serde(default)]
    pub duration: ConditionDuration,
}

/// A condition that is currently active on an actor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveCondition {
    pub kind: ConditionKind,
    /// If true, attempt end-of-turn save each turn using `end_save`.
    pub save_ends_each_turn: bool,
    pub end_phase: Option<DurationPhase>,
    /// For end-of-turn saves, we need the save that ends it.
    pub end_save: Option<SavingThrow>,
    /// Internal flag so a one-turn duration expires exactly once.
    pub pending_one_turn: bool,
}

impl ActiveCondition {
    pub fn from_spec_for_application(spec: &ConditionSpec) -> Self {
        Self {
            kind: spec.kind,
            save_ends_each_turn: spec.duration.save_ends_each_turn,
            end_phase: spec.duration.until,
            end_save: spec.save,
            pending_one_turn: spec.duration.until.is_some(),
        }
    }
}

/// Net vantage result for attack rolls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vantage {
    Normal,
    Advantage,
    Disadvantage,
}

impl Vantage {
    pub fn combine(self, other: Vantage) -> Vantage {
        use Vantage::*;
        match (self, other) {
            (Disadvantage, Advantage) | (Advantage, Disadvantage) => Normal,
            (Normal, x) => x,
            (x, Normal) => x,
            (Advantage, Advantage) => Advantage,
            (Disadvantage, Disadvantage) => Disadvantage,
        }
    }
}

/// Whether the attack is melee or ranged (used for prone interactions).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackStyle {
    Melee,
    Ranged,
}

/// Compute vantage modifiers from conditions on attacker and target.
pub fn vantage_from_conditions(
    attacker_conds: &[ActiveCondition],
    target_conds: &[ActiveCondition],
    style: AttackStyle,
) -> Vantage {
    use ConditionKind::*;
    use Vantage::*;

    let mut net = Normal;

    if attacker_conds
        .iter()
        .any(|c| matches!(c.kind, Poisoned | Restrained))
    {
        net = net.combine(Disadvantage);
    }

    for c in target_conds {
        match c.kind {
            Restrained => {
                net = net.combine(Advantage);
            }
            Prone => match style {
                AttackStyle::Melee => net = net.combine(Advantage),
                AttackStyle::Ranged => net = net.combine(Disadvantage),
            },
            Poisoned => {}
        }
    }

    net
}

/// Lifecycle hooks to expire or allow saves at turn boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnBoundary {
    StartOfTurn,
    EndOfTurn,
}

pub fn process_turn_boundary(
    boundary: TurnBoundary,
    actor_name: &str,
    actor_conds: &mut Vec<ActiveCondition>,
    mut saving_throw_fn: impl FnMut(Ability, i32) -> (i32, i32),
    mut log: impl FnMut(String),
) {
    use TurnBoundary::*;

    if matches!(boundary, EndOfTurn) {
        let mut to_remove = vec![];
        for (idx, c) in actor_conds.iter().enumerate() {
            if c.save_ends_each_turn {
                if let Some(SavingThrow { ability, dc }) = c.end_save {
                    let (roll, total) = saving_throw_fn(ability, dc);
                    let success = total >= dc;
                    log(format!(
                        "[SAVE][{}] makes a {:?} save DC {} vs {:?}: roll={} total={} → {}",
                        actor_name,
                        ability,
                        dc,
                        c.kind,
                        roll,
                        total,
                        if success { "SUCCESS" } else { "FAIL" }
                    ));
                    if success {
                        to_remove.push(idx);
                    }
                }
            }
        }
        for idx in to_remove.into_iter().rev() {
            let removed = actor_conds.remove(idx);
            log(format!(
                "[COND][{}] is no longer {:?}",
                actor_name, removed.kind
            ));
        }
    }

    let phase = match boundary {
        TurnBoundary::StartOfTurn => DurationPhase::StartOfTurn,
        TurnBoundary::EndOfTurn => DurationPhase::EndOfTurn,
    };

    let mut to_remove = vec![];
    for (idx, c) in actor_conds.iter().enumerate() {
        if c.pending_one_turn && c.end_phase == Some(phase) {
            to_remove.push(idx);
        }
    }

    for idx in to_remove.into_iter().rev() {
        let removed = actor_conds.remove(idx);
        log(format!(
            "[COND][{}] {:?} ends at {:?}",
            actor_name, removed.kind, phase
        ));
    }
}

pub fn maybe_apply_on_hit_condition(
    target_name: &str,
    target_conditions: &mut Vec<ActiveCondition>,
    spec: &ConditionSpec,
    mut saving_throw_fn: impl FnMut(Ability, i32) -> (i32, i32),
    mut log: impl FnMut(String),
) {
    if let Some(save) = spec.save {
        let (roll, total) = saving_throw_fn(save.ability, save.dc);
        let success = total >= save.dc;
        log(format!(
            "[SAVE][{}] resists {:?}? {:?} save DC {}: roll={} total={} → {}",
            target_name,
            spec.kind,
            save.ability,
            save.dc,
            roll,
            total,
            if success { "RESISTED" } else { "FAILED" }
        ));
        if success {
            return;
        }
    }

    let active = ActiveCondition::from_spec_for_application(spec);
    target_conditions.push(active);
    log(format!("[COND][{}] gains {:?}", target_name, spec.kind));
}
