use jni::objects::{JClass, JString};
use jni::sys::{jint, jlong};
use jni::JNIEnv;

#[no_mangle]
pub extern "system" fn Java_com_solo5e_Ffi_version<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> JString<'local> {
    env.new_string("solo5e-ffi 0.1.0").expect("new_string failed")
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
