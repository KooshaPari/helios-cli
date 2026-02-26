#!/usr/bin/env bash
set -euo pipefail

REPO=""
ONCE=0
SLEEP_SECONDS=240
RETRY_WINDOW_SECONDS=1800
AUTO_PING=1
MAX_STATES=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      REPO="$2"
      shift 2
      ;;
    --once)
      ONCE=1
      shift
      ;;
    --sleep)
      SLEEP_SECONDS="$2"
      shift 2
      ;;
    --no-ping)
      AUTO_PING=0
      shift
      ;;
    --max-cycles)
      MAX_STATES="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'HELP'
Usage: scripts/babysit-open-prs.sh [--repo org/repo] [--once] [--sleep SECONDS] [--no-ping] [--max-cycles N]

Default behavior: run in a loop, scan open PRs, classify blockers, and optionally ping
@coderabbitai review when only CodeRabbit is failing due to rate-limit signals.

Output columns:
  PR  Branch  MergeState  Mergeable?  CI(fail/nonCoderabbit/rate)  Status
HELP
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -z "$REPO" ]]; then
  if ! REPO=$(gh repo view --json nameWithOwner -q '.nameWithOwner' 2>/dev/null); then
    remote_url=$(git config --get remote.origin.url || true)
    if [[ -z "$remote_url" ]]; then
      echo "Unable to infer repo. Provide --repo or configure GitHub CLI auth." >&2
      exit 1
    fi

    if [[ "$remote_url" == git@github.com:* ]]; then
      REPO="${remote_url#git@github.com:}"
      REPO="${REPO%.git}"
    elif [[ "$remote_url" == https://github.com/* ]]; then
      REPO="${remote_url#https://github.com/}"
      REPO="${REPO%.git}"
    else
      echo "Unable to infer repo from remote URL: $remote_url" >&2
      exit 1
    fi
  fi
fi

if [[ -z "$REPO" ]]; then
  echo "Could not determine repository. Use --repo org/repo." >&2
  exit 1
fi

ME=$(gh api user -q '.login')
CYCLES=0

status_badge() {
  local merge_state="$1"
  local fail_total="$2"
  local fail_non_cb="$3"
  local cr_rate="$4"

  if [[ "$fail_total" == "0" && "$merge_state" == "CLEAN" ]]; then
    echo "READY"
    return
  fi
  if [[ "$fail_total" -gt 0 && "$fail_non_cb" -eq 0 && "$cr_rate" -gt 0 ]]; then
    echo "BLOCKED-CODERABBIT"
    return
  fi
  if [[ "$fail_total" -gt 0 ]]; then
    echo "BLOCKED-CI"
    return
  fi
  if [[ "$merge_state" != "CLEAN" ]]; then
    echo "BLOCKED-MERGE"
    return
  fi
  echo "BLOCKED"
}

recent_my_request() {
  local pr="$1"
  local cutoff
  cutoff=$(date -u -d "-$RETRY_WINDOW_SECONDS seconds" +%Y-%m-%dT%H:%M:%SZ)

  local recent
  recent=$(gh pr view "$pr" --repo "$REPO" --json comments -q "[.comments[] | select(.author.login == \"$ME\" and .body == \"@coderabbitai review\" and .createdAt >= \"$cutoff\") ] | length")
  [[ "$recent" != "0" ]]
}

ping_coderabbit_if_needed() {
  local pr="$1"
  local fail_total="$2"
  local fail_non_cb="$3"
  local cr_rate="$4"

  if [[ "$AUTO_PING" -eq 0 ]]; then
    return
  fi
  if [[ "$fail_total" -eq 0 || "$fail_non_cb" -ne 0 || "$cr_rate" -eq 0 ]]; then
    return
  fi
  if recent_my_request "$pr"; then
    return
  fi

  if gh pr comment "$pr" --body "@coderabbitai review" >/dev/null; then
    echo "    └ re-triage sent to CodeRabbit"
  fi
}

classify_cycle() {
  local pr_json="$1"
  local num branch merge_state
  num=$(echo "$pr_json" | jq -r '.number')
  branch=$(echo "$pr_json" | jq -r '.headRefName')

  local checks
  checks=$(gh pr checks "$num" --repo "$REPO" --json name,state 2>/dev/null || echo '[]')
  local fail_total fail_non_coderabbit
  fail_total=$(echo "$checks" | jq '[.[] | select(.state=="FAILURE" or .state=="ACTION_REQUIRED")] | length')
  fail_non_coderabbit=$(echo "$checks" | jq '[.[] | select((.state=="FAILURE" or .state=="ACTION_REQUIRED") and (.name | ascii_downcase) != "coderabbit")] | length')

  local comment_json
  comment_json=$(gh pr view "$num" --json comments --repo "$REPO")
  local cr_rate
  cr_rate=$(echo "$comment_json" | jq '[.comments[] | select(.author.login=="coderabbitai" and (.body | test("rate limit|secondary rate limit|quota|retry-after|abuse"; "i")))] | length')

  local meta
  meta=$(gh pr view "$num" --repo "$REPO" --json mergeStateStatus,mergeable)
  merge_state=$(echo "$meta" | jq -r '.mergeStateStatus')
  local mergeable
  mergeable=$(echo "$meta" | jq -r '.mergeable')

  local state
  state=$(status_badge "$merge_state" "$fail_total" "$fail_non_coderabbit" "$cr_rate")

  printf '%-4s | %-24s | %-16s | %-7s | %-5s / %-12s / %-3s | %s\n' \
    "#$num" "${branch:0:24}" "$merge_state" "$mergeable" "$fail_total" "$fail_non_coderabbit" "$cr_rate" "$state"

  if [[ "$state" == BLOCKED-CODERABBIT* ]]; then
    ping_coderabbit_if_needed "$num" "$fail_total" "$fail_non_coderabbit" "$cr_rate"
  fi
}

while true; do
  CYCLES=$((CYCLES + 1))
  ts=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
  echo "[$ts] open PR sweep #$CYCLES"
  echo "PR   | branch                    | mergeState       | mergeable | CI(fail/other/rate) | state"
  echo "--------------------------------------------------------------------------------"

  open_json=$(gh pr list --repo "$REPO" --state open --json number,title,headRefName,mergeStateStatus,mergeable --jq '.[]')

  if [[ -z "$open_json" ]]; then
    echo "No open PRs in $REPO"
  else
    while IFS= read -r pr_json; do
      [[ -z "$pr_json" ]] && continue
      classify_cycle "$pr_json"
    done <<<"$open_json"
  fi

  if [[ "$ONCE" -eq 1 ]]; then
    break
  fi
  if [[ "$MAX_STATES" -gt 0 && "$CYCLES" -ge "$MAX_STATES" ]]; then
    break
  fi

  echo "sleep ${SLEEP_SECONDS}s"
  sleep "$SLEEP_SECONDS"
done
