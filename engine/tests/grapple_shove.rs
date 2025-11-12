use engine::combat::actions::{attempt_grapple, attempt_shove_prone};
use engine::conditions::{attempt_escape_grapple_end_of_turn, ActiveCondition, ConditionKind};

fn d20_seq(seq: &[i32]) -> impl FnMut() -> i32 {
    let mut i = 0usize;
    let values = seq.to_vec();
    move || {
        let result = values[i % values.len()];
        i += 1;
        result
    }
}

#[test]
fn grapple_applies_grappled_on_win() {
    let mut conds = vec![];
    let mut logs = vec![];
    let ok = attempt_grapple(
        "Hero",
        3,
        "Gob",
        1,
        2,
        &mut conds,
        d20_seq(&[15, 10]),
        |s| logs.push(s),
    );
    assert!(ok);
    assert!(conds.iter().any(|c| c.kind == ConditionKind::Grappled));
}

#[test]
fn shove_sets_prone_on_win() {
    let mut conds = vec![];
    let mut logs = vec![];
    let ok = attempt_shove_prone("Hero", 3, "Gob", 1, 1, &mut conds, d20_seq(&[12, 5]), |s| {
        logs.push(s)
    });
    assert!(ok);
    assert!(conds.iter().any(|c| c.kind == ConditionKind::Prone));
}

#[test]
fn escape_grapple_removes_condition() {
    let mut conds = vec![ActiveCondition {
        kind: ConditionKind::Grappled,
        save_ends_each_turn: false,
        end_phase: None,
        end_save: None,
        pending_one_turn: false,
    }];
    let mut logs = vec![];
    attempt_escape_grapple_end_of_turn("Gob", 1, 2, 0, &mut conds, d20_seq(&[10, 15]), |s| {
        logs.push(s)
    });
    assert!(!conds.iter().any(|c| c.kind == ConditionKind::Grappled));
}
