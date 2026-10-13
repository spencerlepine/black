# From Rust to iPhone: Getting Your App onto a Real Device

*You've built an app in Rust. It runs on your laptop. Now you want it on your phone, and on your teammates' phones too. This guide covers that last step.*

---

## The short version

Apple gives you two ways to get an app onto a physical iPhone without the App Store:

| | **Path A: Xcode direct install** | **Path B: Signed IPA + install link** |
|---|---|---|
| Best for | Your own phone, quick iteration | Sharing with teammates |
| Device must be plugged into your Mac | Yes, once | No |
| Developer Mode required | Yes | No |
| One-click install for others | No (needs UDID registration) | Yes, via Safari link or QR code |
| Signing | Xcode, using the AMZN Mobile LLC team | AWS CDSigner (Amazon enterprise cert) |
| How long it lasts | About 1 year (team profile); 7 days with a free Apple ID | Until the enterprise cert rotates |

Start with **Path A** to get it running on your own phone, then use **Path B** once other people need it.

---

## Step 0: Put your Rust code inside an iOS app

iOS only runs `.app` bundles, so your Rust code needs a thin native app around it.

**Easy option:** if you used Tauri, Dioxus, or `cargo-mobile2`, it already generates an Xcode project for you (`cargo tauri ios init`, `dx bundle --platform ios`, `cargo mobile init`). Skip ahead.

**Manual option:** build Rust as a static library and link it into a SwiftUI app.

```bash
# In Cargo.toml:  [lib] crate-type = ["staticlib"]
rustup target add aarch64-apple-ios aarch64-apple-ios-sim

cargo build --release --target aarch64-apple-ios       # physical device
cargo build --release --target aarch64-apple-ios-sim   # simulator (Apple Silicon)

xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libmyapp.a     -headers include/ \
  -library target/aarch64-apple-ios-sim/release/libmyapp.a -headers include/ \
  -output MyRustCore.xcframework
```

Generate C headers with `cbindgen`, or let UniFFI generate Swift bindings. Then create a new iOS App project in Xcode, drag in `MyRustCore.xcframework`, and call into it from Swift.

---

## Path A: Install directly from Xcode

### 1. Sign in with the team account
Go to Xcode → **Settings → Accounts** and add the Apple ID that has the **AMZN Mobile LLC** developer role. This is the same role the [AmazonConnectChatIOS README][readme] requires.

> A free personal Apple ID also works, but the app expires after **7 days** and you're limited to 3 apps.

### 2. Configure signing
Select your app target, open **Signing & Capabilities**, then:
- **Team:** AMZN Mobile LLC
- **Bundle Identifier:** something unique, e.g. `com.amazon.<alias>.myrustapp`
- Check **Automatically manage signing**

### 3. Prepare the phone
1. Plug the iPhone into your Mac and tap **Trust This Computer**.
2. On the phone, turn on **Settings → Privacy & Security → Developer Mode** and restart when prompted. The toggle only appears after the phone has been connected to Xcode once.
3. Once it restarts, confirm that you want to enable Developer Mode.

### 4. Run it
Pick your phone as the run destination in Xcode and press **⌘R**. Xcode registers the device's UDID with the team, builds, signs, and installs the app. The app stays on the phone after you unplug it.

### Adding a teammate on Path A
Each device has to be registered on the team. Either plug it into your Mac once and run from Xcode, or add its UDID in the Apple Developer portal and regenerate the profile. Each phone also needs Developer Mode turned on. There's no one-click install on this path; Path B covers that.

---

## Path B: Signed IPA with a one-click install link

The [AmazonConnectChatIOS][readme] package includes scripts that sign an app with **AWS CDSigner** (Amazon's enterprise certificate). An enterprise-signed app installs on any iPhone from a Safari link, with no UDID registration and no Developer Mode.

> These scripts are written for the ChatIOS **Example App**. For your own app, change the workspace, scheme, and bundle ID in `scripts/App/step-1-generate-ipa.sh`. Ask the [iOS SDK quip][ios-sdk-quip] owners whether CDSigner needs your new bundle ID allowlisted.

### 1. One-time setup
```bash
brazil ws create --root MobileAppBuild && cd ~/workplace/MobileAppBuild/src
brazil ws use -p AmazonConnectChatIOS && cd AmazonConnectChatIOS

chmod +x scripts/*.sh scripts/SDK/*.sh
cd scripts && source ./setEnvVariables.sh
mwinit            # CDSigner needs fresh Midway credentials (may need -f)
```

### 2. Build and sign the IPA
```bash
./scripts/App/step-1-generate-ipa.sh
```
This builds the app, signs it with CDSigner, uploads it to S3, and downloads the signed `.ipa`.

### 3. Host it and generate the manifest
1. Upload the signed `.ipa` to an S3 bucket the phones can reach, and copy the object URL.
2. Put that URL into `scripts/step-2-generate-public-plist.sh` and run it to produce a `.plist` manifest.
3. Upload the `.plist` and copy its URL.

### 4. Share the install link
```
itms-services://?action=download-manifest&url=<plist-file-url>
```
Send the link, or turn it into a QR code. Teammates should:
1. Open the link in **Safari** on their iPhone (other browsers won't work) and tap **Install**.
2. Go to **Settings → General → VPN & Device Management** and trust the Amazon developer certificate.
3. Launch the app.

### Path B cautions
- **The bucket is public.** Anyone with the link can download the IPA. Don't bake in secrets or credentials, and delete the objects when you're done.
- **Enterprise distribution is for employees only.** Don't share the link outside Amazon.
- **Clean up between builds:** `rm -rf output/`. You don't need to commit anything unless you changed the scripts.

---

## Troubleshooting

| Symptom | Fix |
|---|---|
| Developer Mode toggle is missing | Connect the phone to Xcode once, then check Settings again. |
| "Untrusted Developer" on launch | Settings → General → VPN & Device Management → Trust. |
| Can't enable Developer Mode or install enterprise apps on a work phone | MDM policy may block it. Try a personal device. |
| App stopped opening after a week | It was signed with a free Apple ID. Re-sign with the AMZN Mobile LLC team. |
| `xcodebuild` hangs resolving packages | `rm -rf ~/Library/Caches/org.swift.swiftpm ~/Library/Developer/Xcode/DerivedData`, then restart (from the [README][readme]). |
| CDSigner step fails | Re-run `mwinit -f`, then check that your bundle ID is allowed. |

---

## Resources

- [AmazonConnectChatIOS README][readme]: the signing scripts, IPA generation, and install-link steps this guide is based on
- [Mobile SDK Release Checklist (Pippin)][pippin]: release prerequisites and manual testing scenarios
- [iOS SDK setup & release (Quip)][ios-sdk-quip]: background on how the iOS SDK signing setup works
- [Push notifications for the iOS Demo App (Quip)][push-quip]: if your app needs APNs

[readme]: https://code.amazon.com/packages/AmazonConnectChatIOS/blobs/mainline/--/README.md
[pippin]: https://pippin.amazon.dev/docs/KDLmzLaw8Umysc/mobile-sdk-release-checklist
[ios-sdk-quip]: https://quip-amazon.com/SzrtOPsKjW9M/iOS-SDK
[push-quip]: https://quip-amazon.com/47JbAnE8zS9E/Setting-up-Notifications-with-iOSDemoApp
