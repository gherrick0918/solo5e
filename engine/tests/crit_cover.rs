use engine::{attack, damage, AdMode, Cover, DamageDice, Dice};

#[test]
fn crit_on_kept_20_without_advantage() {
    let mut dice = Dice::from_scripted(vec![20]);
    let res = attack(&mut dice, AdMode::Normal, 5, 10);
    assert!(res.is_crit);
    assert_eq!(res.raw_rolls, vec![20]);
    assert_eq!(res.roll, 20);
    assert!(res.hit);
}

#[test]
fn crit_on_kept_20_with_advantage() {
    let mut dice = Dice::from_scripted(vec![7, 20]);
    let res = attack(&mut dice, AdMode::Advantage, 5, 10);
    assert!(res.is_crit);
    assert_eq!(res.raw_rolls, vec![7, 20]);
    assert_eq!(res.roll, 20);
}

#[test]
fn no_crit_when_twenty_is_dropped_with_disadvantage() {
    let mut dice = Dice::from_scripted(vec![20, 7]);
    let res = attack(&mut dice, AdMode::Disadvantage, 5, 10);
    assert!(!res.is_crit);
    assert_eq!(res.raw_rolls, vec![20, 7]);
    assert_eq!(res.roll, 7);
}

#[test]
fn cover_bonuses_are_applied() {
    assert_eq!(Cover::None.ac_bonus(), 0);
    assert_eq!(Cover::Half.ac_bonus(), 2);
    assert_eq!(Cover::ThreeQuarters.ac_bonus(), 5);
}

#[test]
fn crit_damage_doubles_dice_only() {
    let dd = DamageDice::new(1, 8);

    let mut normal_dice = Dice::from_scripted(vec![4]);
    let normal = damage(&mut normal_dice, dd, 3, false);
    assert_eq!(normal - 3, 4);

    let mut crit_dice = Dice::from_scripted(vec![4, 5]);
    let crit = damage(&mut crit_dice, dd, 3, true);
    assert_eq!(crit - 3, 9);
}
