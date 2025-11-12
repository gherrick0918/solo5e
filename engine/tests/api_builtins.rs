use engine::api::{
    simulate_duel, simulate_duel_many, simulate_encounter, DuelConfig, EncounterConfig,
};

#[test]
fn duel_with_builtins_runs() {
    let cfg = DuelConfig {
        target_id: Some("poison_goblin".into()),
        weapons_id: Some("basic".into()),
        target_path: None,
        weapons_path: None,
        weapon: "longsword".into(),
        actor_conditions: vec![],
        enemy_conditions: vec![],
        seed: 2025,
        actor_hp: Some(12),
    };
    let res = simulate_duel(cfg).unwrap();
    assert!(res.rounds > 0);
}

#[test]
fn duel_many_summary_makes_sense() {
    let cfg = DuelConfig {
        target_id: Some("poison_goblin".into()),
        weapons_id: Some("basic".into()),
        target_path: None,
        weapons_path: None,
        weapon: "longsword".into(),
        actor_conditions: vec![],
        enemy_conditions: vec![],
        seed: 1,
        actor_hp: Some(12),
    };
    let stats = simulate_duel_many(cfg, 50).unwrap();
    assert_eq!(stats.samples, 50);
    assert_eq!(stats.actor_wins + stats.enemy_wins + stats.draws, 50);
}

#[test]
fn encounter_with_builtins_runs() {
    let cfg = EncounterConfig {
        encounter_id: Some("goblin_ambush".into()),
        encounter_path: None,
        seed: 4242,
        actor_hp: Some(10),
        actor_conditions: vec![],
    };
    let res = simulate_encounter(cfg).unwrap();
    assert!(res.rounds > 0);
}
