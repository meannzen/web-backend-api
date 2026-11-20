#!/usr/bin/env bash
# Seeds local dev data for testing the user list API (role / created_at filters).
#
#   1. Registers an admin (POST /auth/register) and promotes it via SQL.
#   2. Logs in and creates TOTAL users (POST /users, admin only).
#   3. Via SQL, spreads the seeded users over roles and over the last 30 days.
#
# Requires: curl, jq, and either psql or `docker compose` (service `db`).
# The API rate-limits at ~5 req/s per IP, so seeding is sequential and throttled.
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:8080}"
TOTAL="${TOTAL:-50}"
ADMIN_EMAIL="${ADMIN_EMAIL:-admin@seed.local}"
ADMIN_PASSWORD="${ADMIN_PASSWORD:-password123}"
USER_PASSWORD="${USER_PASSWORD:-password123}"
DATABASE_URL="${DATABASE_URL:-postgres://web-app:web-app@127.0.0.1:5432/web-app}"
DELAY="${DELAY:-0.25}"

command -v curl >/dev/null || { echo "curl is required" >&2; exit 1; }
command -v jq >/dev/null || { echo "jq is required" >&2; exit 1; }

run_sql() {
  if command -v psql >/dev/null; then
    psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -c "$1"
  else
    docker compose exec -T db sh -c \
      'psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 -q -c "$1"' sh "$1"
  fi
}

# POST json, retrying on 429. Prints "<status> <body>" on one line.
post() {
  local path="$1" body="$2" auth="${3:-}" out status attempt=0
  while :; do
    local args=(-s -w $'\n%{http_code}' -X POST "${BASE_URL}${path}" -H "Content-Type: application/json" -d "$body")
    [[ -n "$auth" ]] && args+=(-H "Authorization: Bearer ${auth}")
    out=$(curl "${args[@]}")
    status="${out##*$'\n'}"
    if [[ "$status" == "429" && $attempt -lt 10 ]]; then
      attempt=$((attempt + 1))
      sleep 7
      continue
    fi
    printf '%s %s\n' "$status" "${out%$'\n'*}"
    return 0
  done
}

echo "==> Admin ${ADMIN_EMAIL}"
res=$(post /api/v1/auth/register "$(jq -nc --arg e "$ADMIN_EMAIL" --arg p "$ADMIN_PASSWORD" \
  '{email:$e,password:$p,first_name:"Seed",last_name:"Admin"}')")
case "${res%% *}" in
  201) echo "registered" ;;
  409) echo "already exists" ;;
  *) echo "register failed: $res" >&2; exit 1 ;;
esac
run_sql "UPDATE users SET role='admin' WHERE email='${ADMIN_EMAIL}';"

res=$(post /api/v1/auth/login "$(jq -nc --arg e "$ADMIN_EMAIL" --arg p "$ADMIN_PASSWORD" \
  '{email:$e,password:$p}')")
[[ "${res%% *}" == "200" ]] || { echo "login failed: $res" >&2; exit 1; }
TOKEN=$(jq -r .token <<<"${res#* }")

echo "==> Creating ${TOTAL} users"
success=0 failed=0
for i in $(seq 1 "$TOTAL"); do
  email="user${i}@seed.local"
  res=$(post /api/v1/users "$(jq -nc --arg e "$email" --arg p "$USER_PASSWORD" --arg n "User${i}" \
    '{email:$e,password:$p,first_name:$n,last_name:"Seed"}')" "$TOKEN")
  case "${res%% *}" in
    201|409) success=$((success + 1)) ;;
    *) failed=$((failed + 1)); echo "FAIL ${email}: ${res}" >&2 ;;
  esac
  sleep "$DELAY"
done

echo "==> Spreading roles and created_at (users *@seed.local, excluding ${ADMIN_EMAIL})"
run_sql "
WITH numbered AS (
  SELECT id, row_number() OVER (ORDER BY email) AS n
  FROM users
  WHERE email LIKE 'user%@seed.local'
)
UPDATE users u SET
  role = (ARRAY['teacher','teacher','teacher','principal','hr'])[(n % 5) + 1],
  created_at = NOW() - (n * INTERVAL '14 hours')
FROM numbered WHERE u.id = numbered.id;"

echo ""
echo "Done — created/existing: ${success}, failed: ${failed}"
echo "Admin login: ${ADMIN_EMAIL} / ${ADMIN_PASSWORD}"
echo "Try: GET ${BASE_URL}/api/v1/users?role=hr&created_at_gt=$(date -u -v-7d +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -d '7 days ago' +%Y-%m-%dT%H:%M:%SZ)"
[[ $failed -eq 0 ]]
