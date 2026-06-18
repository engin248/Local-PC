#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== 1/5 npm run check ==="
npm run check

echo "=== 2/5 vitest ==="
npx vitest run

echo "=== 3/5 cargo test ==="
(cd src-tauri && cargo test)

echo "=== 4/5 vite build ==="
npm run build
test -f build/brain_logo.png
test -f build/tauri.svg
test -f ONAYLI_TUM_ISLEMLER.cmd
test -f scripts/onayli_tum_islemler.ps1

echo "=== 5/5 config yollar ==="
test -f config/yerel_veri_yollari.json
test -f config/yerel_calisma_modu.json
test -f static/config/voice_persona.json

echo ""
echo "TUM DOGRULAMA GECTI"
