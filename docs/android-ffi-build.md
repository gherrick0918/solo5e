# Android FFI Build & Test Guide

This project ships a minimal JNI bridge that exposes three functions from Rust to Kotlin:

- `version(): String`
- `roll(seed: Long, n: Int, sides: Int): Int`
- `echoJsonLen(json: String): Int`

Follow the steps below to rebuild the shared library and run the smoke tests on an emulator or device.

## Prerequisites

Install [`cargo-ndk`](https://github.com/bbqsrc/cargo-ndk):

```bash
cargo install cargo-ndk
```

Ensure you have the Android SDK, the NDK, and an emulator/device available for instrumented tests.

## Build the native library

From the repo root:

```bash
cargo ndk -t arm64-v8a -t x86_64 -o app/src/main/jniLibs build -p ffi --release
```

This produces the following artifacts:

```
app/src/main/jniLibs/arm64-v8a/libffi.so
app/src/main/jniLibs/x86_64/libffi.so
```

For a device-only build you can target just `arm64-v8a`:

```bash
cargo ndk -t arm64-v8a -o app/src/main/jniLibs build -p ffi --release
```

## Instrumented tests

Pick an x86_64 emulator for the first run:

```bash
./gradlew clean :app:connectedDebugAndroidTest
```

To test on a physical device:

1. Enable USB debugging and connect the device.
2. Verify it appears with `adb devices`.
3. Install the debug build and run tests:

   ```bash
   ./gradlew installDebug
   ./gradlew connectedDebugAndroidTest
   ```

## Manual smoke test (optional)

If you wire the `FfiSmokeUI` composable into a debug screen you can manually trigger `Ffi.roll()` and check the toast output.
