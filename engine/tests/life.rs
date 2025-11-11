use engine::life::*;

fn noop_log(_: String) {}

#[test]
fn nat20_wakes_to_one_hp() {
    let mut h = Health {
        hp: 0,
        max_hp: 10,
        state: LifeState::Unconscious { stable: false },
        death: Default::default(),
    };
    let outcome = process_death_save_start_of_turn("Hero", &mut h, || 20, noop_log);
    assert_eq!(h.state, LifeState::Conscious);
    assert_eq!(h.hp, 1);
    assert!(outcome.is_some());
}

#[test]
fn nat1_counts_two_failures_and_can_kill() {
    let mut h = Health {
        hp: 0,
        max_hp: 10,
        state: LifeState::Unconscious { stable: false },
        death: DeathSaves {
            successes: 0,
            failures: 1,
        },
    };
    let _ = process_death_save_start_of_turn("Hero", &mut h, || 1, noop_log);
    assert!(matches!(h.state, LifeState::Dead));
}

#[test]
fn three_successes_stabilize() {
    let mut h = Health {
        hp: 0,
        max_hp: 10,
        state: LifeState::Unconscious { stable: false },
        death: DeathSaves {
            successes: 2,
            failures: 0,
        },
    };
    let _ = process_death_save_start_of_turn("Hero", &mut h, || 10, noop_log);
    assert!(matches!(h.state, LifeState::Unconscious { stable: true }));
}

#[test]
fn healing_resets_death_saves_and_wakes() {
    let mut h = Health {
        hp: 0,
        max_hp: 12,
        state: LifeState::Unconscious { stable: false },
        death: DeathSaves {
            successes: 2,
            failures: 2,
        },
    };
    heal("Hero", &mut h, 6, noop_log);
    assert_eq!(h.hp, 6);
    assert_eq!(h.death.successes, 0);
    assert_eq!(h.death.failures, 0);
    assert_eq!(h.state, LifeState::Conscious);
}

#[test]
fn apply_damage_triggers_unconscious_and_prone_once() {
    use engine::conditions::{ActiveCondition, ConditionKind};
    let mut h = Health {
        hp: 3,
        max_hp: 10,
        state: LifeState::Conscious,
        death: Default::default(),
    };
    let mut conds: Vec<ActiveCondition> = vec![];
    let mut seen = vec![];
    let dropped = apply_damage("Hero", &mut h, &mut conds, 5, |s| seen.push(s));
    assert!(dropped);
    assert_eq!(h.hp, 0);
    assert!(matches!(h.state, LifeState::Unconscious { stable: false }));
    assert!(conds.iter().any(|c| c.kind == ConditionKind::Prone));
}
