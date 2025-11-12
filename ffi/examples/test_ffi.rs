// Inline the LCG roller for testing
fn roll_test(seed: i64, n: i32, sides: i32) -> i32 {
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

fn main() {
    println!("FFI Version: solo5e-ffi 0.1.0");

    // Test roll function
    let result = roll_test(42, 3, 6);
    println!("roll_test(42, 3, 6) = {}", result);

    // Test with different parameters
    let result2 = roll_test(2025, 1, 20);
    println!("roll_test(2025, 1, 20) = {}", result2);

    // Test edge cases
    println!("roll_test(42, 0, 6) = {}", roll_test(42, 0, 6));
    println!("roll_test(42, 1, 1) = {}", roll_test(42, 1, 1));

    // Test determinism
    println!("Determinism check:");
    println!(
        "  First call:  roll_test(999, 2, 10) = {}",
        roll_test(999, 2, 10)
    );
    println!(
        "  Second call: roll_test(999, 2, 10) = {}",
        roll_test(999, 2, 10)
    );
}
