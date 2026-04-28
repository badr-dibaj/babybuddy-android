# BabyBuddy Android

A fully native Android baby-tracking app written in **Rust** with **Slint** as the UI framework
and **SQLite** (via rusqlite) for local, offline storage. Inspired by the open-source
[BabyBuddy](https://github.com/babybuddy/babybuddy) project.

## Features

| Feature | Description |
|---|---|
| 👶 Babies | Manage multiple babies with name, birthdate, optional photo |
| 🍼 Feedings | Log breast / bottle / solid feedings with duration & amount |
| 🩹 Diapers | Log diaper changes (wet / solid / both) with color & notes |
| ⚖️ Weight | Track weight over time with a growth chart |
| 📏 Height | Track height over time |
| 🔵 Head circumference | Track head circumference |
| 🦷 Teeth | Record tooth eruption (20 primary teeth) |
| 💊 Medications | Log medications with dose and unit |
| 🏠 Dashboard | Daily summary: feedings, diapers, last events, latest measurements |

Everything runs **100% on-device** — no server, no cloud, no internet required.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust (stable, edition 2021) |
| UI | [Slint](https://slint.dev) 1.9 (declarative, compiled, GPU-accelerated) |
| Database | SQLite via [rusqlite](https://crates.io/crates/rusqlite) with bundled feature |
| Platform | Android 10+ (API 29), NativeActivity |
| Build | Cargo workspace + Gradle wrapper |

---

## Project Structure

```
babybuddy-android/
├── Cargo.toml                     # Workspace manifest
├── .cargo/
│   └── config.toml               # Android cross-compilation linker config
├── app/
│   ├── Cargo.toml                # App crate — cdylib for Android
│   ├── build.rs                  # Slint compile step
│   ├── ui/
│   │   └── app.slint             # Complete Slint UI (Material You design)
│   └── src/
│       ├── lib.rs                # android_main entry point + callback wiring
│       ├── db/
│       │   ├── mod.rs            # Database struct + all CRUD operations
│       │   ├── schema.rs         # SQL DDL (CREATE TABLE statements)
│       │   └── models.rs         # Rust data model structs + helpers
│       └── ui/
│           └── mod.rs            # UI helper functions (time formatting, etc.)
└── android/
    ├── settings.gradle
    ├── build.gradle              # Root Gradle file
    ├── gradle.properties
    └── app/
        ├── build.gradle          # App module — triggers `cargo build`, copies .so
        └── src/main/
            ├── AndroidManifest.xml
            ├── java/com/babybuddy/
            │   └── MainActivity.kt   # Thin NativeActivity wrapper
            └── res/values/
                ├── strings.xml
                └── styles.xml
```

---

## Prerequisites

### 1. Rust toolchain

```bash
# Install rustup if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add Android targets
rustup target add aarch64-linux-android    # arm64 — modern devices
rustup target add armv7-linux-androideabi  # armv7 — older 32-bit devices
rustup target add x86_64-linux-android    # x86_64 — emulators
```

### 2. Android NDK

Install via Android Studio → SDK Manager → SDK Tools → **NDK (Side by side)**.
Or download directly from https://developer.android.com/ndk/downloads.

Then export the path:

```bash
# Linux
export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/25.2.9519653

# macOS
export ANDROID_NDK_HOME=$HOME/Library/Android/sdk/ndk/25.2.9519653
```

> If your NDK path differs, also update `.cargo/config.toml` to match the correct
> `prebuilt/linux-x86_64` or `prebuilt/darwin-x86_64` prefix.

### 3. Android SDK + JDK

- Android SDK with API 34 platform
- JDK 17+ (bundled with Android Studio)

---

## Building

### Option A — Gradle (recommended, builds Rust + Android APK)

```bash
cd android

# Debug APK
./gradlew assembleDebug

# Release APK
./gradlew assembleRelease
```

The Gradle build will automatically invoke `cargo build` for each ABI,
copy the resulting `libbabybuddy.so` into `jniLibs/`, and then package the APK.

The APK will be at `android/app/build/outputs/apk/debug/app-debug.apk`.

### Option B — Cargo only (compile the Rust library)

```bash
# From the workspace root:
cargo build --target aarch64-linux-android --package babybuddy
# .so at: target/aarch64-linux-android/debug/libbabybuddy.so
```

---

## Installing on a device

```bash
# Enable USB debugging on your Android device, then:
adb install android/app/build/outputs/apk/debug/app-debug.apk
```

---

## Running on an emulator

```bash
# Start an x86_64 AVD from Android Studio, then:
cargo build --target x86_64-linux-android --package babybuddy
# Then build via Gradle:
cd android && ./gradlew assembleDebug
adb install app/build/outputs/apk/debug/app-debug.apk
```

---

## Development Tips

### Live-reload during development

Slint supports hot-reloading `.slint` files on desktop. For rapid UI iteration,
you can create a desktop binary target:

```bash
# Add a [[bin]] target in app/Cargo.toml pointing to a src/main.rs
# that calls AppWindow::new() and runs the Slint loop normally.
cargo run --target x86_64-unknown-linux-gnu
```

### Database inspection

The SQLite database is stored at `/data/data/com.babybuddy.app/babybuddy.db` on device.
Pull it for inspection:

```bash
adb shell run-as com.babybuddy.app cat /data/data/com.babybuddy.app/babybuddy.db > /tmp/bb.db
sqlite3 /tmp/bb.db ".tables"
```

---

## Architecture Notes

- **No JNI bridge**: The app uses `android-activity` with `NativeActivity`, so the Rust
  code owns the entire event loop. Kotlin's `MainActivity.kt` is a 10-line stub.
- **Slint generates the Rust API**: `build.rs` calls `slint_build::compile("ui/app.slint")`
  which generates strongly-typed Rust structs/callbacks for every property and callback
  declared in the `.slint` file. These are included via `slint::include_modules!()`.
- **Thread safety**: The `Database` is wrapped in `Arc<Mutex<>>` and all Slint callbacks
  run on the main thread, so there is no data race risk in the current design.
- **Offline-first**: All data is stored locally in SQLite with WAL mode for performance.

---

## License

MIT — see [LICENSE](LICENSE).

Inspired by [BabyBuddy](https://github.com/babybuddy/babybuddy) (BSD-2-Clause).
