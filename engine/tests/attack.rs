use engine::{attack, damage, AdMode, AttackResult, DamageDice, Dice};

#[test]
fn attack_flags_and_logic_are_self_consistent() {
    let mut dice = Dice::from_seed(777);
    let ac = 15;
    let bonus = 5;
    let res: AttackResult = attack(&mut dice, AdMode::Normal, bonus, ac);

    // flags match roll
    assert_eq!(res.nat20, res.roll == 20);
    assert_eq!(res.nat1, res.roll == 1);

    // hit logic = nat20 OR (!nat1 AND total >= ac)
    let expected_hit = res.nat20 || (!res.nat1 && res.total >= res.ac);
    assert_eq!(res.hit, expected_hit);
}

#[test]
fn damage_roll_is_within_bounds() {
    let mut dice = Dice::from_seed(42);
    let dd = DamageDice::new(2, 6); // 2d6

    let noncrit = damage(&mut dice, dd, 3, false);
    // min 2..12 then +3 => 5..15
    assert!(noncrit >= 5 && noncrit <= 15);

    let mut dice2 = Dice::from_seed(42);
    let crit = damage(&mut dice2, dd, 3, true);
    // crit doubles dice: 4..24 then +3 => 7..27
    assert!(crit >= 7 && crit <= 27);
}
