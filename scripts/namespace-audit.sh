#!/usr/bin/env bash
set -euo pipefail

REPO=""
NAMESPACE="KooshaPari"
ALLOWED_BOTS="github-actions[bot],dependabot[bot],dependabot-preview[bot],renovate[bot],app/github-actions"
CLOSE_OFFENDERS=0
DO_COMMENT=1
DRY_RUN=0
SCAN_PRS=1
SCAN_ISSUES=1
JSON_REPORT=""
REDIRECT_TEMPLATE=""

usage() {
  cat <<'USAGE'
Usage:
  scripts/namespace-audit.sh --repo org/repo [options]

Options:
  --repo <org/repo>               Repository to scan.
  --namespace <owner>              Allowed owner namespace.
  --close                          Close offending items after commenting.
  --no-comment                     Skip comment. Default: comment enabled.
  --only-prs                       Only scan open pull requests.
  --only-issues                    Only scan open issues.
  --dry-run                        Report offenses without making mutations.
  --allowed-bots <a,b,c>           Comma-separated bot allowlist.
  --redirect-template <text>        Custom redirect message.
  --json-report <path>             Write JSON report to this file.
  --help                           Show this help.
USAGE
}

require_cmds() {
  local missing=0
  for cmd in gh jq; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
      echo "Missing required command: $cmd" >&2
      missing=1
    fi
  done
  if [[ "$missing" -ne 0 ]]; then
    exit 1
  fi
}

infer_repo_from_remote() {
  local remote_url
  remote_url=$(git config --get remote.origin.url || true)
  if [[ -z "$remote_url" ]]; then
    return 1
  fi

  if [[ "$remote_url" == git@github.com:* ]]; then
    REPO="${remote_url#git@github.com:}"
    REPO="${REPO%.git}"
    return 0
  fi

  if [[ "$remote_url" == https://github.com/* ]]; then
    REPO="${remote_url#https://github.com/}"
    REPO="${REPO%.git}"
    return 0
  fi

  return 1
}

is_author_allowed() {
  local author=$1
  [[ "$author" == "$NAMESPACE" ]] && return 0

  local bot
  IFS=',' read -r -a bot <<< "$ALLOWED_BOTS"
  for allowed in "${bot[@]}"; do
    [[ -z "$allowed" ]] && continue
    [[ "$author" == "$allowed" ]] && return 0
  done

  return 1
}

is_head_owner_allowed() {
  local owner=$1
  [[ -n "$owner" && "$owner" == "$NAMESPACE" ]]
}

default_template() {
  cat <<TEMPLATE
This item was created outside the allowed namespace policy.

Allowed namespace: **${NAMESPACE}**

To keep this workstream clean, please open this PR/issue in a KooshaPari-owned repository.

If this item was created accidentally, close and recreate it in one of the KooshaPari repos,
and link back to this reference for continuity.
TEMPLATE
}

comment_and_close_pr() {
  local number=$1
  local author=$2
  local title=$3
  local url=$4

  echo "Offending PR: #$number by @$author - $title"
  echo "  -> $url"

  if [[ "$DO_COMMENT" -eq 1 && "$DRY_RUN" -eq 0 ]]; then
    gh pr comment "$number" --repo "$REPO" --body "$REDIRECT_TEMPLATE" >/dev/null
  fi

  if [[ "$CLOSE_OFFENDERS" -eq 1 && "$DRY_RUN" -eq 0 ]]; then
    gh pr close "$number" --repo "$REPO" >/dev/null
    echo "  -> closed"
  fi
}

comment_and_close_issue() {
  local number=$1
  local author=$2
  local title=$3
  local url=$4

  echo "Offending issue: #$number by @$author - $title"
  echo "  -> $url"

  if [[ "$DO_COMMENT" -eq 1 && "$DRY_RUN" -eq 0 ]]; then
    gh issue comment "$number" --repo "$REPO" --body "$REDIRECT_TEMPLATE" >/dev/null
  fi

  if [[ "$CLOSE_OFFENDERS" -eq 1 && "$DRY_RUN" -eq 0 ]]; then
    gh issue close "$number" --repo "$REPO" >/dev/null
    echo "  -> closed"
  fi
}

run_scan() {
  local offenders_json='[]'
  local offender_count=0

  if [[ "$SCAN_PRS" -eq 1 ]]; then
    local prs_json
    prs_json=$(gh api --paginate --slurp "repos/$REPO/pulls?state=open&per_page=100")
    if [[ -z "$prs_json" ]] || ! jq -e . >/dev/null 2>&1 <<<"$prs_json"; then
      echo "ERROR: invalid PR JSON from gh pr list" >&2
      return 1
    fi

    while IFS= read -r item; do
      [[ -z "$item" ]] && continue

      local number title author url head_owner reason=""
      number=$(jq -r '.number // 0' <<<"$item")
      title=$(jq -r '.title // ""' <<<"$item")
      url=$(jq -r '.html_url // ""' <<<"$item")
      author=$(jq -r '.user.login // ""' <<<"$item")
      head_owner=$(jq -r '.head.repo.owner.login // ""' <<<"$item")

      local bad=0
      local -a reasons=()
      if ! is_author_allowed "$author"; then
        bad=1
        reasons+=("author")
      fi
      if ! is_head_owner_allowed "$head_owner"; then
        bad=1
        reasons+=("source-repo-owner")
      fi
      local reason
      reason=$(IFS=,; echo "${reasons[*]}")

      if [[ "$bad" -eq 1 ]]; then
        offender_count=$((offender_count + 1))
        offenders_json=$(
          printf '%s' "$offenders_json" | jq --compact-output \
            --arg number "$number" \
            --arg author "$author" \
            --arg reason "$reason" \
            --arg url "$url" \
            '. + [{type:"pr", number: ($number|tonumber), author: $author, reason: $reason, url: $url}]'
        )

        if [[ "$DRY_RUN" -eq 1 ]]; then
          echo "[DRY-RUN] would enforce PR #$number by @$author ($reason)"
        else
          comment_and_close_pr "$number" "$author" "$title" "$url"
        fi
      fi
    done < <(jq -c '.[][]' <<<"$prs_json")
  fi

  if [[ "$SCAN_ISSUES" -eq 1 ]]; then
    local issues_json
    issues_json=$(gh api --paginate --slurp "repos/$REPO/issues?state=open&per_page=100")
    if [[ -z "$issues_json" ]] || ! jq -e . >/dev/null 2>&1 <<<"$issues_json"; then
      echo "ERROR: invalid issue JSON from gh issue list" >&2
      return 1
    fi

    while IFS= read -r item; do
      [[ -z "$item" ]] && continue
      if jq -e '.pull_request' <<<"$item" >/dev/null 2>&1; then
        continue
      fi

      local number title author url reason
      number=$(jq -r '.number // 0' <<<"$item")
      title=$(jq -r '.title // ""' <<<"$item")
      url=$(jq -r '.html_url // ""' <<<"$item")
      author=$(jq -r '.user.login // ""' <<<"$item")
      reason="author"

      if ! is_author_allowed "$author"; then
        offender_count=$((offender_count + 1))
        offenders_json=$(
          printf '%s' "$offenders_json" | jq --compact-output \
            --arg number "$number" \
            --arg author "$author" \
            --arg reason "$reason" \
            --arg url "$url" \
            '. + [{type:"issue", number: ($number|tonumber), author: $author, reason: $reason, url: $url}]'
        )

        if [[ "$DRY_RUN" -eq 1 ]]; then
          echo "[DRY-RUN] would enforce issue #$number by @$author"
        else
          comment_and_close_issue "$number" "$author" "$title" "$url"
        fi
      fi
    done < <(jq -c '.[][]' <<<"$issues_json")
  fi

  if [[ -n "$JSON_REPORT" ]]; then
    jq -n \
      --arg repo "$REPO" \
      --arg namespace "$NAMESPACE" \
      --argjson scan_prs "$SCAN_PRS" \
      --argjson scan_issues "$SCAN_ISSUES" \
      --argjson total "$offender_count" \
      --arg generated "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
      --argjson offenders "$offenders_json" \
      '{repository:$repo, namespace:$namespace, scan_pulls: $scan_prs,
        scan_issues:$scan_issues, total_offenders:$total,
        generated_at:$generated, offenders:$offenders}' \
      > "$JSON_REPORT"
  fi

  echo "Total namespace violations: $offender_count"
}

if [[ "$#" -eq 0 ]]; then
  usage
  exit 1
fi

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --repo)
      REPO="$2"
      shift 2
      ;;
    --namespace)
      NAMESPACE="$2"
      shift 2
      ;;
    --close)
      CLOSE_OFFENDERS=1
      shift
      ;;
    --no-comment)
      DO_COMMENT=0
      shift
      ;;
    --only-prs)
      SCAN_PRS=1
      SCAN_ISSUES=0
      shift
      ;;
    --only-issues)
      SCAN_PRS=0
      SCAN_ISSUES=1
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --allowed-bots)
      ALLOWED_BOTS="$2"
      shift 2
      ;;
    --redirect-template)
      REDIRECT_TEMPLATE="$2"
      shift 2
      ;;
    --json-report)
      JSON_REPORT="$2"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      usage
      exit 1
      ;;
  esac
done

require_cmds

if [[ -z "$REPO" ]]; then
  if ! infer_repo_from_remote; then
    echo "Unable to infer repository from remote. Use --repo." >&2
    exit 1
  fi
fi

if [[ -z "$REDIRECT_TEMPLATE" ]]; then
  REDIRECT_TEMPLATE="$(default_template)"
fi

run_scan
