#!/usr/bin/env bash
# After a release: set the version and checksums in the Homebrew formula and
# the Scoop manifest from the release's SHA256SUMS. The release workflow runs
# this and pushes the results to baoulo/homebrew-tap and baoulo/scoop-bucket.
#
#   scripts/update-packaging.sh 0.2.0
#   SUMS_FILE=SHA256SUMS scripts/update-packaging.sh 0.2.0
set -euo pipefail

VERSION="${1:?usage: $0 <version>}"
VERSION="${VERSION#v}"
TAG="v${VERSION}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FORMULA="${ROOT}/packaging/homebrew/ditch.rb"
SCOOP="${ROOT}/packaging/scoop/ditch.json"

sums="$(mktemp)"
trap 'rm -f "${sums}"' EXIT
if [[ -n "${SUMS_FILE:-}" ]]; then
  cp "${SUMS_FILE}" "${sums}"
else
  curl -fsSL "https://github.com/baoulo/ditch/releases/download/${TAG}/SHA256SUMS" -o "${sums}"
fi

sum_for() {
  local file="ditch-${TAG}-$1"
  local sum
  sum="$(awk -v f="${file}" '$2 == f || $2 == "*" f { print $1 }' "${sums}")"
  if [[ ! "${sum}" =~ ^[0-9a-f]{64}$ ]]; then
    echo "error: no SHA-256 for ${file} in SHA256SUMS" >&2
    exit 1
  fi
  echo "${sum}"
}

tmp="$(mktemp)"
awk -v version="${VERSION}" \
  -v mac_arm="$(sum_for macos-aarch64.tar.gz)" -v mac_intel="$(sum_for macos-x86_64.tar.gz)" \
  -v linux_arm="$(sum_for linux-aarch64.tar.gz)" -v linux_intel="$(sum_for linux-x86_64.tar.gz)" '
  /^  version "/ { print "  version \"" version "\""; next }
  pending != "" { sub(/^ *#? *sha256 .*/, "      sha256 \"" pending "\""); print; pending = ""; next }
  /url .*macos-aarch64/ { pending = mac_arm }
  /url .*macos-x86_64/ { pending = mac_intel }
  /url .*linux-aarch64/ { pending = linux_arm }
  /url .*linux-x86_64/ { pending = linux_intel }
  { print }
' "${FORMULA}" > "${tmp}"
mv "${tmp}" "${FORMULA}"

win="$(sum_for windows-x86_64.zip)"
sed -i.bak -E \
  -e "s/^  \"version\": \"[^\"]*\"/  \"version\": \"${VERSION}\"/" \
  -e "s#releases/download/v[0-9][^/]*/ditch-v[0-9][^\"]*-windows-x86_64.zip#releases/download/${TAG}/ditch-${TAG}-windows-x86_64.zip#" \
  -e "s/\"hash\": \"[^\"]*\"/\"hash\": \"${win}\"/" \
  "${SCOOP}"
rm -f "${SCOOP}.bak"

echo "Updated for ${TAG}:"
echo "  ${FORMULA#"${ROOT}/"}"
echo "  ${SCOOP#"${ROOT}/"}"
