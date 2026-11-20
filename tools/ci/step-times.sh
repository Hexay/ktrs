#!/usr/bin/env bash
# Per-step wall time of a workflow's last N successful runs, slowest first.
# Usage: tools/ci/step-times.sh <workflow.yml> [runs=3] [top=25]
set -euo pipefail
wf=$1 runs=${2:-3} top=${3:-25}
ids=$(gh run list -w "$wf" -s success -L "$runs" --json databaseId --jq '.[].databaseId')
[ -n "$ids" ] || { echo "no successful runs of $wf" >&2; exit 1; }
for id in $ids; do
  gh run view "$id" --json jobs --jq '.jobs[] | .name as $j
    | ((((.completedAt|fromdate)-(.startedAt|fromdate))) as $t | "\($j)\t(job total)\t\($t)"),
      (.steps[] | select(.startedAt and .completedAt)
        | "\($j)\t\(.name)\t\(((.completedAt|fromdate)-(.startedAt|fromdate)))")'
done | awk -F'\t' -v n="$runs" '{k=$1"\t"$2; s[k]+=$3; c[k]++; if($3>m[k])m[k]=$3}
  END{for(k in s) printf "%6.1f %6.1f\t%s\n", s[k]/c[k]/60, m[k]/60, k}' \
  | sort -rn | head -n "$top" | sed '1i avg_min max_min\tjob\tstep'
