use engine::conditions::{
    process_turn_boundary, vantage_from_conditions, ActiveCondition, AttackStyle, ConditionKind,
    TurnBoundary, Vantage,
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
