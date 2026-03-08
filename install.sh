#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SYSTEMD=0
for arg in "$@"; do
  case "$arg" in
    --systemd) SYSTEMD=1 ;;
    *) echo "unknown arg: $arg" >&2; exit 1 ;;
  esac
done

cd "$REPO_ROOT"
echo "[clawhip] install flow: clone -> install.sh -> SKILL attach -> config scaffold -> daemon start -> live verification"
echo "[clawhip] repo root: $REPO_ROOT"
cargo install --path . --force
mkdir -p "$HOME/.clawhip"
echo "[clawhip] ensured config dir $HOME/.clawhip"
echo "[clawhip] next: read SKILL.md and attach the skill surface"

if [[ "$SYSTEMD" == "1" ]]; then
  sudo cp deploy/clawhip.service /etc/systemd/system/clawhip.service
  sudo systemctl daemon-reload
  sudo systemctl enable --now clawhip
  echo "[clawhip] systemd unit installed and started"
fi

echo "[clawhip] recommended verification: scripts/live-verify-default-presets.sh <mode>"
echo "[clawhip] install complete"
