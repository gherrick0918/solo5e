use crate::Ability;

/// Result of a contested check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContestOutcome {
    AttackerWins,
    DefenderWins,
    TieDefender,
}

/// Roll contested d20 + mod; ties go to defender.
pub fn contested_check(
    mut d20: impl FnMut() -> i32,
    att_mod: i32,
    def_mod: i32,
    mut log: impl FnMut(String),
    att_label: &str,
    def_label: &str,
) -> ContestOutcome {
    let ar = d20();
    let dr = d20();
    let at = ar + att_mod;
    let dt = dr + def_mod;
    log(format!(
        "[CONTEST] {} d20={} ({} total) vs {} d20={} ({} total)",
        att_label, ar, at, def_label, dr, dt
    ));
    if at > dt {
        ContestOutcome::AttackerWins
    } else if at == dt {
        ContestOutcome::TieDefender
    } else {
        ContestOutcome::DefenderWins
    }
}

/// Choose defender's best of STR or DEX.
pub fn best_of_str_dex(str_mod: i32, dex_mod: i32) -> (Ability, i32) {
    if dex_mod > str_mod {
        (Ability::Dex, dex_mod)
    } else {
        (Ability::Str, str_mod)
    }
}
