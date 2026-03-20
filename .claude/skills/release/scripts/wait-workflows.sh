#!/usr/bin/env bash
# Wait for CI and Release workflows to complete.
# Exits 0 on success, 1 on timeout (still in progress), 2 on failure.
# Usage: bash wait-workflows.sh [max_iterations] [sleep_interval_seconds]
#
# Designed to fit within Claude Code's 10-minute Bash timeout:
#   default 8 iterations x 60s = max 8 minutes per call.
# If it exits 1 (timeout), re-run the script to continue waiting.

REPO="fuchigta/cc-launcher"
MAX=${1:-8}
INTERVAL=${2:-60}

for i in $(seq 1 "$MAX"); do
  CI=$(gh run list --repo "$REPO" --workflow=ci.yml --limit=1 --json status,conclusion --jq '.[0]')
  REL=$(gh run list --repo "$REPO" --workflow=release.yml --limit=1 --json status,conclusion --jq '.[0]')

  CI_STATUS=$(echo "$CI" | jq -r '.status')
  REL_STATUS=$(echo "$REL" | jq -r '.status')
  CI_RESULT=$(echo "$CI" | jq -r '.conclusion // "—"')
  REL_RESULT=$(echo "$REL" | jq -r '.conclusion // "—"')

  echo "[$i/$MAX] CI=$CI_STATUS($CI_RESULT) Release=$REL_STATUS($REL_RESULT)"

  if [ "$CI_STATUS" = "completed" ] && [ "$REL_STATUS" = "completed" ]; then
    if [ "$CI_RESULT" = "success" ] && [ "$REL_RESULT" = "success" ]; then
      echo "done: both workflows succeeded"
      exit 0
    else
      echo "done: failure detected (CI=$CI_RESULT Release=$REL_RESULT)"
      exit 2
    fi
  fi

  [ "$i" -lt "$MAX" ] && sleep "$INTERVAL"
done

echo "timeout: workflows still in progress — re-run this script to continue"
exit 1
