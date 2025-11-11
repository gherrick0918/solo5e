# Android FFI build & test cheatsheet

## Prerequisites

Install [`cargo-ndk`](https://github.com/bbqsrc/cargo-ndk):

```bash
cargo install cargo-ndk
```

Ensure the Android SDK/NDK and an emulator or device are available for Gradle's
instrumented tests.

## Build the JNI library

From the repo root, build the `ffi` crate for both common ABIs (emulator +
device) and drop the results into the app's `jniLibs` directory:

```bash
cargo ndk -t arm64-v8a -t x86_64 -o app/src/main/jniLibs build -p ffi --release
```

You should see the following artifacts afterward:

```
app/src/main/jniLibs/arm64-v8a/libffi.so
app/src/main/jniLibs/x86_64/libffi.so
```

For a phone-only workflow you can build just arm64:

```bash
cargo ndk -t arm64-v8a -o app/src/main/jniLibs build -p ffi --release
```

## Run Android instrumented tests

Use Gradle to drive the instrumentation suite (ensure an emulator/device is
connected):

```bash
./gradlew clean :app:connectedDebugAndroidTest
```

For a real device you can also install the debug build before testing:

```bash
./gradlew installDebug
./gradlew connectedDebugAndroidTest
```

If `adb devices` does not list your hardware/emulator as `device`, resolve the
connection before retrying the tests.
