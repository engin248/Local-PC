#!/usr/bin/env bash
# Cloud Agent: tunel URL ile Windows kopru cagrisi
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CONFIG="$ROOT/config/kopru_bridge.json"
OP="${1:-health}"
BASE_URL="${KOPRU_TUNEL_URL:-}"
TOKEN="${KOPRU_TOKEN:-}"

if [[ -z "$BASE_URL" ]] && [[ -f "$CONFIG" ]]; then
  BASE_URL="$(python3 -c "import json; c=json.load(open('$CONFIG')); print(c.get('tunnel',{}).get('tunnel_public_url','').rstrip('/'))" 2>/dev/null || true)"
  TOKEN="$(python3 -c "import json; c=json.load(open('$CONFIG')); print(c.get('token',''))" 2>/dev/null || true)"
fi

if [[ -z "$BASE_URL" ]]; then
  echo "HATA: tunnel_public_url yok. Windows'ta KOPRU_TUNEL_BASLAT.cmd calistirin, URL'yi config/kopru_bridge.json'a yazin."
  exit 1
fi

case "$OP" in
  health)
    curl -fsS "$BASE_URL/v1/health"
    ;;
  yol_kontrol|kopru_durum)
    curl -fsS -H "X-Kopru-Token: $TOKEN" "$BASE_URL/v1/paths"
    ;;
  kurulu_guncelle|panel_kapat)
    curl -fsS -X POST -H "X-Kopru-Token: $TOKEN" -H "Content-Type: application/json" \
      -d "{\"op\":\"$OP\"}" "$BASE_URL/v1/run"
    ;;
  *)
    echo "Bilinmeyen op: $OP"
    exit 1
    ;;
esac
echo ""
