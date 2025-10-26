use engine::{Ability, AbilityScores, Actor, AdMode, Dice, Skill};
use std::collections::HashSet;

fn sample_fighter() -> Actor {
    // L1 Fighter example: PB +2, STR/CON saves, Athletics + Perception
    let abilities = AbilityScores {
        str_: 16,
        dex: 14,
        con: 14,
        int_: 10,
        wis: 12,
        cha: 8,
    };
    let mut save = HashSet::new();
    save.insert(Ability::Str);
    save.insert(Ability::Con);
    let mut skills = HashSet::new();
    skills.insert(Skill::Athletics);
    skills.insert(Skill::Perception);
    Actor {
        abilities,
        proficiency_bonus: 2,
        save_proficiencies: save,
        skill_proficiencies: skills,
    }
}

#[test]
fn fighter_mods() {
    let a = sample_fighter();
    // ability mods
    assert_eq!(a.ability_mod(Ability::Str), 3);
    assert_eq!(a.ability_mod(Ability::Dex), 2);
    assert_eq!(a.ability_mod(Ability::Wis), 1);
    // save profs: STR/CON add +2 PB
    assert_eq!(a.save_mod(Ability::Str), 5);
    assert_eq!(a.save_mod(Ability::Con), 4);
    assert_eq!(a.save_mod(Ability::Dex), 2);
    // skill profs: Athletics (STR), Perception (WIS) add +2 PB
    assert_eq!(a.skill_mod(Skill::Athletics), 5);
    assert_eq!(a.skill_mod(Skill::Perception), 3);
}

#[test]
fn fighter_checks_are_deterministic() {
    let a = sample_fighter();
    let mut dice = Dice::from_seed(222);
    let res = a.skill_check(&mut dice, Skill::Athletics, AdMode::Normal, 13);
    assert_eq!(res.passed, res.total >= res.dc);
}
