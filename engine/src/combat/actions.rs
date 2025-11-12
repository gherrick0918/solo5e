use crate::checks::{best_of_str_dex, contested_check, ContestOutcome};
use crate::conditions::{ActiveCondition, ConditionKind};

#[allow(clippy::too_many_arguments)]
pub fn attempt_grapple(
    attacker_name: &str,
    attacker_str_mod: i32,
    defender_name: &str,
    defender_str_mod: i32,
    defender_dex_mod: i32,
    defender_conds: &mut Vec<ActiveCondition>,
    d20: impl FnMut() -> i32,
    mut log: impl FnMut(String),
) -> bool {
    let (_, def_mod) = best_of_str_dex(defender_str_mod, defender_dex_mod);
    match contested_check(
        d20,
        attacker_str_mod,
        def_mod,
        &mut log,
        &format!("{} (STR)", attacker_name),
        &format!("{} (best STR/DEX)", defender_name),
    ) {
        ContestOutcome::AttackerWins => {
            if !defender_conds
                .iter()
                .any(|c| c.kind == ConditionKind::Grappled)
            {
                defender_conds.push(ActiveCondition {
                    kind: ConditionKind::Grappled,
                    save_ends_each_turn: false,
                    end_phase: None,
                    end_save: None,
                    pending_one_turn: false,
                });
            }
            log(format!(
                "[COND][{}] is now Grappled (speed 0)",
                defender_name
            ));
            true
        }
        _ => {
            log("[CONTEST] Grapple fails".to_string());
            false
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn attempt_shove_prone(
    attacker_name: &str,
    attacker_str_mod: i32,
    defender_name: &str,
    defender_str_mod: i32,
    defender_dex_mod: i32,
    defender_conds: &mut Vec<ActiveCondition>,
    d20: impl FnMut() -> i32,
    mut log: impl FnMut(String),
) -> bool {
    let (_, def_mod) = best_of_str_dex(defender_str_mod, defender_dex_mod);
    match contested_check(
        d20,
        attacker_str_mod,
        def_mod,
        &mut log,
        &format!("{} (STR)", attacker_name),
        &format!("{} (best STR/DEX)", defender_name),
    ) {
        ContestOutcome::AttackerWins => {
            if !defender_conds
                .iter()
                .any(|c| c.kind == ConditionKind::Prone)
            {
                defender_conds.push(ActiveCondition {
                    kind: ConditionKind::Prone,
                    save_ends_each_turn: false,
                    end_phase: None,
                    end_save: None,
                    pending_one_turn: false,
                });
            }
            log(format!("[COND][{}] is shoved Prone", defender_name));
            true
        }
        _ => {
            log("[CONTEST] Shove fails".to_string());
            false
        }
    }
}
