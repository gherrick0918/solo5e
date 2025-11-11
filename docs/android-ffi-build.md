# Android FFI Build Notes

1. Install [`cargo-ndk`](https://github.com/bbqsrc/cargo-ndk):
   ```bash
   cargo install cargo-ndk
   ```
2. Build the JNI library for Android emulators (x86_64) and arm64 devices:
   ```bash
   cargo ndk -t arm64-v8a -t x86_64 -o app/src/main/jniLibs build -p ffi --release
   ```
3. Confirm that Gradle picks up the generated shared libraries:
   ```
   ls app/src/main/jniLibs/arm64-v8a/libffi.so
   ls app/src/main/jniLibs/x86_64/libffi.so
   ```
4. Run instrumented tests on an emulator or device:
   ```bash
   ./gradlew clean :app:connectedDebugAndroidTest
   ```
