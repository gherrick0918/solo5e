use engine::api::{simulate_duel, DuelConfig};

#[test]
fn duel_api_smoke() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest.parent().expect("workspace root");
    let target_path = repo_root
        .join("content/targets/poison_goblin.json")
        .to_string_lossy()
        .into_owned();
    let weapons_path = repo_root
        .join("content/weapons/basic.json")
        .to_string_lossy()
        .into_owned();

    let cfg = DuelConfig {
        target_path: Some(target_path),
        weapons_path: Some(weapons_path),
        target_id: None,
        weapons_id: None,
        weapon: "longsword".to_string(),
        actor_conditions: vec![],
        enemy_conditions: vec![],
        seed: 2025,
        actor_hp: Some(12),
    };
    let res = simulate_duel(cfg).expect("duel ran");
    assert!(res.rounds > 0);
    assert!(matches!(res.winner.as_str(), "actor" | "enemy" | "draw"));
    assert!(!res.log.is_empty());
}
