#!/usr/bin/env bash
# Prints one USAGE: line for the account-wide Claude plan limits. See agents/usage-cap-watchdog.md.
set -uo pipefail

# Pay-as-you-go backends (Bedrock, Vertex, Foundry, raw API key) have no plan cap and no usage endpoint.
# Check the env first, then any Claude settings file's "env" block (in case this runs outside Claude Code).
setting() {
  local v=${!1:-}
  [ -n "$v" ] && { printf '%s' "$v"; return; }
  for f in "${CLAUDE_CONFIG_DIR:-$HOME/.claude}/settings.json" "${CLAUDE_CONFIG_DIR:-$HOME/.claude}/settings.local.json" \
           "$(dirname "$0")/../settings.json" "$(dirname "$0")/../settings.local.json"; do
    v=$(jq -r --arg k "$1" '.env[$k] // empty' "$f" 2>/dev/null) && [ -n "$v" ] && { printf '%s' "$v"; return; }
  done
}
truthy() { case "${1,,}" in 1|true|yes|on) return 0;; *) return 1;; esac; }
backend=""
truthy "$(setting CLAUDE_CODE_USE_BEDROCK)" && backend="AWS Bedrock"
truthy "$(setting CLAUDE_CODE_USE_VERTEX)" && backend="Google Vertex AI"
truthy "$(setting CLAUDE_CODE_USE_FOUNDRY)" && backend="Microsoft Foundry"
[ -n "$backend" ] && { echo "USAGE: GREEN 0% — no plan usage cap (Claude Code runs on $backend, billed per token; no session/weekly limits apply)"; exit 0; }

creds=$(security find-generic-password -s "Claude Code-credentials" -w 2>/dev/null \
  || cat "${CLAUDE_CONFIG_DIR:-$HOME/.claude}/.credentials.json" 2>/dev/null)
token=$(printf '%s' "$creds" | jq -r '.claudeAiOauth.accessToken // empty' 2>/dev/null)
[ -z "$token" ] && [ -n "$(setting ANTHROPIC_API_KEY)$(setting ANTHROPIC_AUTH_TOKEN)" ] && {
  echo "USAGE: GREEN 0% — no plan usage cap (Claude Code uses an API key, billed per token; no session/weekly limits apply)"; exit 0; }
[ -z "$token" ] && { echo "USAGE: UNKNOWN — no Claude Code OAuth token found (not logged in with a claude.ai plan?)"; exit 1; }

body=$(curl -sS -m 15 -w '\n%{http_code}' https://api.anthropic.com/api/oauth/usage \
  -H "Authorization: Bearer $token" -H "anthropic-beta: oauth-2025-04-20" 2>&1)
code=${body##*$'\n'}; body=${body%$'\n'*}
[ "$code" = "000" ] && { echo "USAGE: UNKNOWN — usage endpoint unreachable (network error or timeout)"; exit 1; }
[ "$code" != "200" ] && { echo "USAGE: UNKNOWN — usage endpoint returned HTTP $code"; exit 1; }

printf '%s' "$body" | jq -r '
  def color(p): if p >= 90 then "RED" elif p >= 75 then "ORANGE" else "GREEN" end;
  def name: if .kind == "session" then "session(5h)"
            elif .kind == "weekly_all" then "weekly"
            else "weekly-" + (.scope.model.display_name // .kind) end;
  def epoch: if . == null then null else .[0:19] + "Z" | fromdateiso8601 end;
  def zulu: todate | .[0:16] | sub("T"; " ") + "Z";
  def left: (. - now) as $s | if $s <= 0 then "now"
            else ($s / 86400 | floor) as $d | ($s % 86400 / 3600 | floor) as $h | ($s % 3600 / 60 | floor) as $m
            | (if $d > 0 then "\($d)d" else "" end) + "\($h)h\($m)m" end;
  def resets: if .reset == null then "no reset time" else "resets \(.reset | zulu), in \(.reset | left)" end;
  ( (.limits // [
      (.five_hour | select(.) | {kind:"session",    percent:.utilization, resets_at}),
      (.seven_day | select(.) | {kind:"weekly_all", percent:.utilization, resets_at})
    ]) | map(select(.percent != null) | . + {reset: (.resets_at | epoch)}) ) as $l
  | if $l == [] then "USAGE: GREEN 0% — no capped usage limits reported (plan is unlimited)" else
    ([$l[].percent] | max // 0) as $worst
  | ([$l[] | select(.percent >= 75) | .reset // empty] | max) as $last
  | "USAGE: \(color($worst)) \($worst | floor)% — "
    + ([$l[] | "\(name) \(.percent | floor)%\(if .percent >= 100 then " LIMIT HIT" else "" end) (\(resets))"] | join(", "))
    + (if .extra_usage.is_enabled then " | extra usage \(.extra_usage.utilization | floor)% of $\(.extra_usage.monthly_limit / 100)" else "" end)
    + (if $worst < 75 then ""
       elif $last == null then " | resume_at unknown"
       else ((($last + 300 + 59) / 60 | floor) * 60) as $r | ($r | localtime) as $t
         | " | resume_at \($r | zulu) (in \($r | left)) resume_cron \"\($t[4]) \($t[3]) \($t[2]) \($t[1] + 1) *\""
       end)
  end
' 2>/dev/null || { echo "USAGE: UNKNOWN — usage endpoint returned an unreadable response"; exit 1; }
