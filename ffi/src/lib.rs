use engine::api::{
    simulate_duel, simulate_duel_many, simulate_encounter, DuelConfig, EncounterConfig,
};
use jni::objects::{JClass, JString};
use jni::sys::{jint, jlong, jstring};
use jni::JNIEnv;
use serde_json::json;

fn ok(env: &JNIEnv, value: serde_json::Value) -> jstring {
    let payload = json!({ "ok": true, "result": value });
    env.new_string(serde_json::to_string(&payload).unwrap())
        .unwrap()
        .into_raw()
}

fn err(env: &JNIEnv, e: impl std::fmt::Display) -> jstring {
    env.new_string(format!(r#"{{"ok":false,"error":"{}"}}"#, e))
        .unwrap()
        .into_raw()
}

#[no_mangle]
pub extern "system" fn Java_com_solo5e_Ffi_version<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> JString<'local> {
    env.new_string("solo5e-ffi 0.1.0")
        .expect("new_string failed")
}

/// Deterministic roller: sum of n rolls of 1..=sides using a simple LCG.
/// Handles edge cases: n<=0 → 0, sides<=1 → 1 per die.
#[no_mangle]
pub extern "system" fn Java_com_solo5e_Ffi_roll(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    seed: jlong,
    n: jint,
    sides: jint,
) -> jint {
    let mut state = seed as u64;
    let mut next_u32 = || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (state >> 32) as u32
    };
    let rolls = n.max(0) as i64;
    let sides = sides.max(1) as i64;
    let mut total = 0i64;
    for _ in 0..rolls {
        let r = (next_u32() as i64 % sides) + 1; // 1..=sides
        total += r;
    }
    total as jint
}

#[no_mangle]
pub extern "system" fn Java_com_solo5e_Ffi_echoJsonLen<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    json: JString<'local>,
) -> jint {
    let s: String = env.get_string(&json).expect("get_string").into();
    s.len() as jint
}

#[no_mangle]
pub extern "system" fn Java_com_solo5e_Ffi_simulateDuelJson(
    mut env: JNIEnv,
    _class: JClass,
    json: JString,
) -> jstring {
    let input: String = match env.get_string(&json) {
        Ok(s) => s.into(),
        Err(e) => return err(&env, e),
    };
    let cfg: DuelConfig = match serde_json::from_str(&input) {
        Ok(c) => c,
        Err(e) => return err(&env, format!("invalid_config: {}", e)),
    };
    match simulate_duel(cfg) {
        Ok(result) => ok(&env, serde_json::to_value(result).unwrap()),
        Err(e) => err(&env, e),
    }
}

#[no_mangle]
pub extern "system" fn Java_com_solo5e_Ffi_simulateDuelManyJson(
    mut env: JNIEnv,
    _class: JClass,
    json: JString,
) -> jstring {
    let input: String = match env.get_string(&json) {
        Ok(s) => s.into(),
        Err(e) => return err(&env, e),
    };
    let mut root: serde_json::Value = match serde_json::from_str(&input) {
        Ok(v) => v,
        Err(e) => return err(&env, format!("invalid_config: {}", e)),
    };
    let samples = root.get("samples").and_then(|v| v.as_u64()).unwrap_or(100) as u32;
    if let Some(obj) = root.as_object_mut() {
        obj.remove("samples");
    }
    let cfg: DuelConfig = match serde_json::from_value(root) {
        Ok(c) => c,
        Err(e) => return err(&env, format!("invalid_config: {}", e)),
    };
    match simulate_duel_many(cfg, samples) {
        Ok(stats) => ok(&env, serde_json::to_value(stats).unwrap()),
        Err(e) => err(&env, e),
    }
}

#[no_mangle]
pub extern "system" fn Java_com_solo5e_Ffi_simulateEncounterJson(
    mut env: JNIEnv,
    _class: JClass,
    json: JString,
) -> jstring {
    let input: String = match env.get_string(&json) {
        Ok(s) => s.into(),
        Err(e) => return err(&env, e),
    };
    let cfg: EncounterConfig = match serde_json::from_str(&input) {
        Ok(c) => c,
        Err(e) => return err(&env, format!("invalid_config: {}", e)),
    };
    match simulate_encounter(cfg) {
        Ok(result) => ok(&env, serde_json::to_value(result).unwrap()),
        Err(e) => err(&env, e),
    }
}

// Internal functions for testing without JNI overhead
pub fn roll_internal(seed: i64, n: i32, sides: i32) -> i32 {
    let mut state = seed as u64;
    let mut next_u32 = || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (state >> 32) as u32
    };
    let rolls = n.max(0) as i64;
    let sides = sides.max(1) as i64;
    let mut total = 0i64;
    for _ in 0..rolls {
        let r = (next_u32() as i64 % sides) + 1; // 1..=sides
        total += r;
    }
    total as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roll_internal() {
        // Test deterministic rolling
        let result1 = roll_internal(42, 3, 6);
        let result2 = roll_internal(42, 3, 6); // Same seed should give same result
        assert_eq!(result1, result2);
        assert!((3..=18).contains(&result1)); // 3d6 range
    }

    #[test]
    fn test_roll_edge_cases() {
        assert_eq!(roll_internal(42, 0, 6), 0); // No rolls
        assert_eq!(roll_internal(42, 1, 1), 1); // Single-sided die
    }
}
