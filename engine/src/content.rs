use std::collections::HashMap;

pub fn builtin_targets() -> HashMap<&'static str, &'static str> {
    HashMap::from([(
        "poison_goblin",
        include_str!("../content/targets/poison_goblin.json"),
    )])
}

pub fn builtin_weapons() -> HashMap<&'static str, &'static str> {
    HashMap::from([("basic", include_str!("../content/weapons/basic.json"))])
}

pub fn builtin_encounters() -> HashMap<&'static str, &'static str> {
    HashMap::from([(
        "goblin_ambush",
        include_str!("../content/encounters/goblin_ambush.json"),
    )])
}
