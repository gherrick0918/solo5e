use jni::objects::{JClass, JString};
use jni::sys::{jint, jlong};
use jni::JNIEnv;

#[no_mangle]
pub extern "system" fn Java_com_solo5e_Ffi_version(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> JString {
    env.new_string("solo5e-ffi 0.1.0").unwrap()
}

/// Tiny deterministic roller (for smoke tests): roll `n` d`sides` with xoshiro-ish LCG.
#[no_mangle]
pub extern "system" fn Java_com_solo5e_Ffi_roll(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    seed: jlong,
    n: jint,
    sides: jint,
) -> jint {
    let mut state = seed as u64;
    let mut next = || {
        state = state.wrapping_mul(636_413_622_384_679_3005).wrapping_add(1);
        ((state >> 32) as u32) as i64
    };
    let mut total = 0i64;
    let rolls = n.max(0) as i64;
    let sides = sides.max(1) as i64;
    for _ in 0..rolls {
        let r = (next() % sides) + 1;
        total += r;
    }
    total as jint
}

/// Placeholder JSON echo → future hook to your engine sims
#[no_mangle]
pub extern "system" fn Java_com_solo5e_Ffi_echoJsonLen(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    json: JString<'_>,
) -> jint {
    let s: String = env.get_string(&json).unwrap().into();
    s.len() as jint
}
