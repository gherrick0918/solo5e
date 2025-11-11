# Android FFI Build Notes

1. Install [`cargo-ndk`](https://github.com/bbqsrc/cargo-ndk):
   ```bash
   cargo install cargo-ndk
   ```
2. Build the JNI library for Android (example for arm64):
   ```bash
   cargo ndk -t arm64-v8a -o app/src/main/jniLibs build -p ffi --release
   ```
3. The command produces `app/src/main/jniLibs/arm64-v8a/libffi.so`, which Gradle will bundle automatically.
