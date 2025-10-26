use engine::{ability_mod, check, AdMode, CheckInput, Dice};

#[test]
fn ability_mod_rounds_down() {
    assert_eq!(ability_mod(8), -1);
    assert_eq!(ability_mod(9), -1);
    assert_eq!(ability_mod(10), 0);
    assert_eq!(ability_mod(11), 0);
    assert_eq!(ability_mod(12), 1);
}

#[test]
fn deterministic_check_total_consistent() {
    let mut dice = Dice::from_seed(123);
    let res = check(
        &mut dice,
        CheckInput {
            dc: 13,
            modifier: 2,
            mode: AdMode::Normal,
        },
    );
    assert_eq!(res.passed, res.total >= res.dc);
}
