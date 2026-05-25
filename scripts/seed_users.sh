#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:8080}"
TOTAL="${TOTAL:-1000}"
CONCURRENCY="${CONCURRENCY:-10}"

success=0
failed=0
pids=()

create_user() {
  local i="$1"
  local email="user${i}@seed.local"
  local password="password${i}"

  local status
  status=$(curl -s -o /dev/null -w "%{http_code}" \
    -X POST "${BASE_URL}/api/v1/users" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"${email}\",\"password\":\"${password}\"}")

  if [[ "$status" == "201" || "$status" == "409" ]]; then
    echo "OK [${status}] ${email}"
    return 0
  else
    echo "FAIL [${status}] ${email}" >&2
    return 1
  fi
}

export -f create_user
export BASE_URL

echo "Seeding ${TOTAL} users to ${BASE_URL} (concurrency=${CONCURRENCY})..."
echo ""

if command -v parallel &>/dev/null; then
  # GNU parallel for fast concurrent execution
  seq 1 "$TOTAL" | parallel -j "$CONCURRENCY" create_user {}
else
  # Fallback: manual background job batching
  for i in $(seq 1 "$TOTAL"); do
    create_user "$i" &
    pids+=($!)

    if (( ${#pids[@]} >= CONCURRENCY )); then
      for pid in "${pids[@]}"; do
        wait "$pid" && ((success++)) || ((failed++))
      done
      pids=()
    fi
  done

  # Wait for remaining jobs
  for pid in "${pids[@]}"; do
    wait "$pid" && ((success++)) || ((failed++))
  done

  echo ""
  echo "Done — success: ${success}, failed: ${failed}"
fi
