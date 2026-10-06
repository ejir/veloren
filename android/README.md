# Veloren Android (arm64, NativeActivity)

This is an experimental Android packaging path for the Voxygen client. It builds a
standalone, landscape APK for 64-bit ARM devices. The Gradle project is a thin
wrapper around the repository's Rust workspace; it does not replace the desktop
build.

## Build requirements

- JDK 17
- Android SDK Platform 35, Android build tools, CMake 3.22.1, and NDK
  `28.0.13004108`
- The nightly Rust toolchain pinned by `../rust-toolchain`
- Rust target `aarch64-linux-android`
- Gradle 8.11.1 and `cargo-ndk`
- Git LFS assets fetched into the checkout

Install the Rust pieces with:

```sh
rustup target add aarch64-linux-android
cargo install cargo-ndk
```

Set `ANDROID_HOME` (or `ANDROID_SDK_ROOT`) to the SDK directory. Then build and
install a debug APK with:

```sh
cd android
gradle :app:assembleDebug
adb install -r app/build/outputs/apk/debug/app-debug.apk
```

The Rust library is cross-compiled by `cargo ndk` during Gradle's `preBuild`.
The generated APK is arm64-only; x86 emulators and 32-bit devices are not
included in this first target.

## CI release pipeline

Pushing a semantic-version tag such as `android-v0.18.0` runs
`../.github/workflows/android-release.yml`. The workflow builds, verifies, and
attaches a signed arm64 APK and SHA-256 checksum to a GitHub Release. Android's
version name comes from the tag; its version code is derived from the major,
minor, and patch numbers.

Before the first release, create and securely back up a dedicated Android
keystore outside the repository. Add these repository Actions secrets:

- `ANDROID_RELEASE_KEYSTORE_BASE64` — base64-encoded `.jks` file
- `ANDROID_RELEASE_KEYSTORE_PASSWORD`
- `ANDROID_RELEASE_KEY_ALIAS`
- `ANDROID_RELEASE_KEY_PASSWORD`

For example, generate a JKS key and encode it on Linux with:

```sh
keytool -genkeypair -storetype JKS -keystore veloren-release.jks -alias veloren
base64 -w0 veloren-release.jks
```

Never commit the keystore or its passwords. Every Android update must be signed with the same
keystore; loss of the key prevents updating installs made with prior releases.

## Assets and storage

The Gradle task stages the repository `assets/` tree into the APK and creates an
asset index and SHA-256 version marker. On first launch (and after an asset
update), NativeActivity copies those assets into its private app data directory
before Voxygen initializes its asset cache. This costs roughly 450 MB of
additional device storage on top of the APK. The assets stay private to the app;
no broad or external-storage permission is requested.

For Play Store distribution, the large asset payload must move to Play Asset
Delivery or another on-demand asset mechanism rather than remain in the base
APK. The CI release is a signed, arm64 sideload APK for device testing and
direct distribution; it is not Play Store-ready.

## Controls

Menus use the existing touch-aware UI. In a game session, the lower-left area is
a virtual movement stick; swipe on the right side to turn the camera. The
translucent right-side targets are attack, secondary action, jump, interact,
and roll.
Hardware keyboard and gamepad input remain available where Android exposes them.

## Current scope

This is an early porting path, not a release-ready Android client. It targets
arm64 Vulkan/OpenGL-capable devices and inherits the desktop client's network
and server requirements. Graphics quality defaults to low on Android. The APK
has not been built or tested on a device in the current development environment;
Android SDK/NDK and Rust are not installed there.
