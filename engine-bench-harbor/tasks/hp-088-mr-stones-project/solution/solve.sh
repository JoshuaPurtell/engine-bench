#!/usr/bin/env bash
set -euo pipefail

WORKDIR="/app"
# Harbor mounts solution/ to /solution/ (not /oracle/)
SOURCE="/solution/hp_088_mr_stones_project.rs"
DEST="$WORKDIR/src/hp/cards/hp_088_mr_stones_project.rs"

echo "=== Oracle solve.sh ==="
echo "Applying gold solution from $SOURCE to $DEST"

if [ ! -f "$SOURCE" ]; then
  echo "ERROR: Missing oracle solution at $SOURCE" >&2
  exit 1
fi

cp "$SOURCE" "$DEST"
echo "Applied oracle solution successfully"
