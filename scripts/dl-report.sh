#!/usr/bin/env bash
set -euo pipefail

BASE="https://fuzz.jamtoaster.network"
KIND="${1:?usage: $0 <report|session> <session-id>}"
SESSION="${2:?usage: $0 <report|session> <session-id>}"

echo "Paste the signed envelope JSON, then press Enter:"
read -r ENVELOPE

B64=$(echo -n "$ENVELOPE" | base64 -w0)

GRANT=$(curl -sf \
  -X POST \
  "$BASE/api/fuzzing/sessions/$SESSION/download-grant" \
  -H "Content-Type: application/json" \
  -H "Authorization: JAM-Envelope $B64" \
  -d "{\"kind\":\"$KIND\"}")

TOKEN=$(echo "$GRANT" | grep -o '"url":"[^"]*"' | cut -d'"' -f4)

echo "$BASE$TOKEN"
