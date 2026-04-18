#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
requested_platform="${1:-auto}"
package="meeru-ui"
binary="meeru"
workspace_manifest="${repo_root}/Cargo.toml"

version="$(
    awk -F'"' '/^version = "/ { print $2; exit }' "${workspace_manifest}"
)"

if [[ -z "${version}" ]]; then
    echo "failed to read workspace version from ${workspace_manifest}" >&2
    exit 1
fi

detect_platform() {
    case "$(uname -s)" in
    Darwin) echo "macos" ;;
    Linux) echo "linux" ;;
    MINGW* | MSYS* | CYGWIN*) echo "windows" ;;
    *)
        echo "unsupported host platform: $(uname -s)" >&2
        exit 1
        ;;
    esac
}

platform="${requested_platform}"
if [[ "${platform}" == "auto" ]]; then
    platform="$(detect_platform)"
fi

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "missing required command: $1" >&2
        exit 1
    fi
}

run_bundle() {
    local format="$1"
    cargo bundle --release -p "${package}" --bin "${binary}" --format "${format}"
}

create_macos_dmg() {
    local app_path
    app_path="$(find "${repo_root}/target/release/bundle/osx" -maxdepth 1 -name '*.app' -print -quit)"

    if [[ -z "${app_path}" ]]; then
        echo "macOS app bundle was not created" >&2
        exit 1
    fi

    local dmg_path="${repo_root}/target/release/bundle/osx/Meeru-${version}-macos.dmg"
    rm -f "${dmg_path}"
    hdiutil create -volname "Meeru" -srcfolder "${app_path}" -ov -format UDZO "${dmg_path}"
    echo "${dmg_path}"
}

case "${platform}" in
macos)
    require_command cargo
    require_command cargo-bundle
    require_command hdiutil
    run_bundle osx
    create_macos_dmg
    ;;
linux)
    require_command cargo
    require_command cargo-bundle
    run_bundle deb
    run_bundle appimage
    ;;
windows)
    require_command cargo
    require_command cargo-bundle
    run_bundle msi
    ;;
*)
    echo "unsupported package target: ${platform}" >&2
    exit 1
    ;;
esac
