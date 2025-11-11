use engine::conditions::{
    maybe_apply_on_hit_condition, process_turn_boundary, vantage_from_conditions, ActiveCondition,
    AttackStyle, ConditionDuration, ConditionKind, ConditionSpec, DurationPhase, TurnBoundary,
    Vantage,
};
use engine::{Ability, SavingThrow};

#[test]
fn poisoned_gives_attacker_disadvantage() {
    let attacker = vec![ActiveCondition {
        kind: ConditionKind::Poisoned,
        save_ends_each_turn: false,
        end_phase: None,
        end_save: None,
        pending_one_turn: false,
    }];
    let target: Vec<ActiveCondition> = vec![];
    assert_eq!(
        vantage_from_conditions(&attacker, &target, AttackStyle::Melee),
        Vantage::Disadvantage
    );
}

#[test]
fn prone_interactions_melee_and_ranged() {
    let attacker: Vec<ActiveCondition> = vec![];
    let target = vec![ActiveCondition {
        kind: ConditionKind::Prone,
        save_ends_each_turn: false,
        end_phase: None,
        end_save: None,
        pending_one_turn: false,
    }];

    assert_eq!(
        vantage_from_conditions(&attacker, &target, AttackStyle::Melee),
        Vantage::Advantage
    );
    assert_eq!(
        vantage_from_conditions(&attacker, &target, AttackStyle::Ranged),
        Vantage::Disadvantage
    );
}

#[test]
fn poisoned_attacker_vs_restrained_target_cancels() {
    let attacker = vec![ActiveCondition {
        kind: ConditionKind::Poisoned,
        save_ends_each_turn: false,
        end_phase: None,
        end_save: None,
        pending_one_turn: false,
    }];
    let target = vec![ActiveCondition {
        kind: ConditionKind::Restrained,
        save_ends_each_turn: false,
        end_phase: None,
        end_save: None,
        pending_one_turn: false,
    }];

    assert_eq!(
        vantage_from_conditions(&attacker, &target, AttackStyle::Melee),
        Vantage::Normal
    );
}

#[test]
fn one_turn_duration_expires_on_next_boundary() {
    let mut conds = vec![ActiveCondition {
        kind: ConditionKind::Prone,
        save_ends_each_turn: false,
        end_phase: Some(DurationPhase::StartOfTurn),
        end_save: None,
        pending_one_turn: true,
    }];

    let mut logs = Vec::new();
    let log = |msg: String| logs.push(msg);

    process_turn_boundary(
        TurnBoundary::StartOfTurn,
        "Tester",
        &mut conds,
        |_ability, _dc| (0, 0),
        log,
    );

    assert!(conds.is_empty(), "Condition should expire after one turn");
    assert!(
        logs.iter()
            .any(|msg| msg.contains("[COND][Tester] Prone ends at StartOfTurn")),
        "Expected log indicating the condition ended"
    );
}

#[test]
fn prone_ranged_disadvantage_cancels_with_other_advantage() {
    let attacker: Vec<ActiveCondition> = vec![];
    let target = vec![
        ActiveCondition {
            kind: ConditionKind::Prone,
            save_ends_each_turn: false,
            end_phase: None,
            end_save: None,
            pending_one_turn: false,
        },
        ActiveCondition {
            kind: ConditionKind::Restrained,
            save_ends_each_turn: false,
            end_phase: None,
            end_save: None,
            pending_one_turn: false,
        },
    ];

    assert_eq!(
        vantage_from_conditions(&attacker, &target, AttackStyle::Ranged),
        Vantage::Normal
    );
}

#[test]
fn application_save_uses_target_ability() {
    let mut conds = Vec::new();
    let spec = ConditionSpec {
        kind: ConditionKind::Poisoned,
        save: Some(SavingThrow {
            ability: Ability::Con,
            dc: 12,
        }),
        duration: ConditionDuration::default(),
    };

    let mut captured = Vec::new();
    maybe_apply_on_hit_condition(
        "Target",
        &mut conds,
        &spec,
        |ability, dc| {
            captured.push((ability, dc));
            (1, 1)
        },
        |_msg| {},
    );

    assert_eq!(captured, vec![(Ability::Con, 12)]);
}

#[test]
fn save_ends_each_turn_drops_condition_when_dc_zero() {
    let mut conds = vec![ActiveCondition {
        kind: ConditionKind::Poisoned,
        save_ends_each_turn: true,
        end_phase: None,
        end_save: Some(SavingThrow {
            ability: Ability::Con,
            dc: 0,
        }),
        pending_one_turn: false,
    }];

    let mut logs = Vec::new();
    let log = |msg: String| logs.push(msg);
    let save = |_ability: Ability, _dc: i32| (15, 15);

    process_turn_boundary(TurnBoundary::EndOfTurn, "Tester", &mut conds, save, log);

    assert!(
        conds.is_empty(),
        "Condition should be removed on successful end-of-turn save"
    );
}
