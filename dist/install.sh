#!/usr/bin/env bash
# NALA-Cezar-RS v0.21.0-nala.6.6.6 — Cryptographically Verified Multi-Arch Installer (macOS / Linux)
set -euo pipefail

REPO="Master-MD/nala-cezar-rs"
VERSION="${NALA_CEZAR_VERSION:-v0.21.0-nala.6.6.6}"
INSTALL_DIR="${NALA_INSTALL_DIR:-$HOME/.local/bin}"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "${OS}-${ARCH}" in
  darwin-arm64|darwin-aarch64) ASSET="nala-cezar-rs-macos-arm64" ;;
  darwin-x86_64)               ASSET="nala-cezar-rs-macos-x86_64" ;;
  linux-x86_64|linux-amd64)    ASSET="nala-cezar-rs-linux-x86_64" ;;
  linux-aarch64|linux-arm64)   ASSET="nala-cezar-rs-linux-aarch64" ;;
  *)
    echo "[ERROR] Unsupported platform: ${OS}-${ARCH}" >&2
    exit 1
    ;;
esac

mkdir -p "${INSTALL_DIR}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
echo "[NALA-Cezar-RS] Downloading ${ASSET} (${VERSION})..."
curl -fsSL "${BASE_URL}/${ASSET}" -o "${TMP_DIR}/${ASSET}"
curl -fsSL "${BASE_URL}/SHA256SUMS.txt" -o "${TMP_DIR}/SHA256SUMS.txt"

echo "[NALA-Cezar-RS] Verifying FIPS 180-4 SHA-256 digest..."
EXPECTED="$(grep "  ${ASSET}\$" "${TMP_DIR}/SHA256SUMS.txt" | awk '{print $1}')"
if command -v sha256sum >/dev/null 2>&1; then
  ACTUAL="$(sha256sum "${TMP_DIR}/${ASSET}" | awk '{print $1}')"
else
  ACTUAL="$(shasum -a 256 "${TMP_DIR}/${ASSET}" | awk '{print $1}')"
fi

if [ -z "${EXPECTED}" ] || [ "${EXPECTED}" != "${ACTUAL}" ]; then
  echo "[FATAL] SHA-256 mismatch! Expected ${EXPECTED}, got ${ACTUAL}" >&2
  exit 1
fi

install -m 0755 "${TMP_DIR}/${ASSET}" "${INSTALL_DIR}/nala-cezar-rs"
ln -sf "${INSTALL_DIR}/nala-cezar-rs" "${INSTALL_DIR}/cez-rs"
echo "[OK] Installed ${INSTALL_DIR}/nala-cezar-rs (alias: cez-rs)"
"${INSTALL_DIR}/nala-cezar-rs" --version
