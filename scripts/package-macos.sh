#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
profile=${1:-release}
case "$profile" in
  debug) cargo_args="" ;;
  release) cargo_args="--release" ;;
  *) echo "usage: $0 [debug|release]" >&2; exit 2 ;;
esac

cd "$root"
cargo build -p luma-ui --locked $cargo_args
target_dir=${CARGO_TARGET_DIR:-target}
case "$target_dir" in
  /*) ;;
  *) target_dir="$root/$target_dir" ;;
esac
app="$root/dist/Luma.app"
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources/ThirdPartyLicenses"
cp "$target_dir/$profile/luma" "$app/Contents/MacOS/luma"
cp "$root/THIRD_PARTY_LICENSES.md" "$app/Contents/Resources/ThirdPartyLicenses/README.md"
cp "$root/LICENSES/Apache-2.0-Zed.txt" "$app/Contents/Resources/ThirdPartyLicenses/Apache-2.0-Zed.txt"
cat > "$app/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleDisplayName</key><string>Luma</string>
  <key>CFBundleExecutable</key><string>luma</string>
  <key>CFBundleIdentifier</key><string>dev.lec.luma</string>
  <key>CFBundleName</key><string>Luma</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>0.1.0</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
plutil -lint "$app/Contents/Info.plist"
codesign --force --deep --sign - "$app"
codesign --verify --deep --strict "$app"
echo "$app"
