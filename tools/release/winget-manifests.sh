#!/usr/bin/env bash
# Writes the winget manifests (Hexay.ktrs) for a release into <out>, from its SHA256SUMS. Run by the release
# workflow, which submits them to microsoft/winget-pkgs when WINGET_TOKEN is set:
#   tools/release/winget-manifests.sh v0.5.0 SHA256SUMS out
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
tag=$1 sums=$2 out=$3
# shellcheck source=release-assets.sh
source "$here/release-assets.sh"
id=Hexay.ktrs version=${tag#v} schema=1.6.0
mkdir -p "$out"

installer() {
  local arch=$1 target=$2 bin
  echo "- Architecture: $arch"
  echo "  InstallerUrl: $(asset_url "$target")"
  echo "  InstallerSha256: $(asset_sha "$target" | tr a-f A-F)"
  echo "  NestedInstallerFiles:"
  for bin in ktrs ktfmt ktlint; do
    echo "  - RelativeFilePath: ktrs-$tag-$target\\$bin.exe"
    echo "    PortableCommandAlias: $bin"
  done
}

cat > "$out/$id.yaml" <<EOF
# yaml-language-server: \$schema=https://aka.ms/winget-manifest.version.$schema.schema.json
PackageIdentifier: $id
PackageVersion: $version
DefaultLocale: en-US
ManifestType: version
ManifestVersion: $schema
EOF

cat > "$out/$id.installer.yaml" <<EOF
# yaml-language-server: \$schema=https://aka.ms/winget-manifest.installer.$schema.schema.json
PackageIdentifier: $id
PackageVersion: $version
InstallerType: zip
NestedInstallerType: portable
Commands:
- ktrs
- ktfmt
- ktlint
Installers:
$(installer x64 x86_64-pc-windows-msvc)
$(installer arm64 aarch64-pc-windows-msvc)
ManifestType: installer
ManifestVersion: $schema
EOF

cat > "$out/$id.locale.en-US.yaml" <<EOF
# yaml-language-server: \$schema=https://aka.ms/winget-manifest.defaultLocale.$schema.schema.json
PackageIdentifier: $id
PackageVersion: $version
PackageLocale: en-US
Publisher: Hexay
PublisherUrl: https://github.com/Hexay
PackageName: ktrs
PackageUrl: https://github.com/Hexay/ktrs
License: MIT OR Apache-2.0
LicenseUrl: https://github.com/Hexay/ktrs/blob/master/LICENSE-MIT
ShortDescription: Fast Kotlin formatter and linter, with drop-in ktfmt and ktlint executables
Tags:
- kotlin
- formatter
- linter
- ktfmt
- ktlint
ReleaseNotesUrl: https://github.com/Hexay/ktrs/releases/tag/$tag
ManifestType: defaultLocale
ManifestVersion: $schema
EOF
