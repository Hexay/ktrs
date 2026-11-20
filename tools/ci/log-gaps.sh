#!/usr/bin/env bash
# Where a job's time goes: the longest gaps between consecutive log lines (the line printed after each gap).
# Usage: tools/ci/log-gaps.sh <job-id> [top=20]   (job ids: gh run view <run> --json jobs --jq '.jobs[]|"\(.databaseId) \(.name)"')
set -euo pipefail
job=$1 top=${2:-20}
gh run view --job "$job" --log | cut -f3- |
  sed -e 's/^\xEF\xBB\xBF//' -e 's/\x1b\[[0-9;]*m//g' |
  awk 'match($0, /^[0-9-]+T([0-9]+):([0-9]+):([0-9.]+)Z /, t) {
         s = t[1]*3600 + t[2]*60 + t[3]
         if (prev != "" && s - prev > 5) printf "%7.1fs  %s\n", s - prev, substr(substr($0, RLENGTH+1), 1, 160)
         prev = s; if (!first) first = s; last = s }
       END { printf "%7.1fs  (total)\n", last - first }' |
  sort -rn | head -n "$top"
