#!/usr/bin/env bash
# Runs one gate of .github/workflows/parity.yml: output to target/parity/<name>/log.txt, the report files matching GLOB
# copied next to it (the job's failure artifact), and a job-summary entry that, on failure, shows the log's last lines
# and each report's first lines. Exits with the gate's status.
#
#   tools/parity/gate.sh NAME GLOB CMD [ARG...]     GLOB: repo-relative, '' for none
set -uo pipefail
(($# >= 3)) || { sed -n 2,6p "$0"; exit 2; }
name=$1 glob=$2; shift 2
root="$(cd "$(dirname "$0")/../.." && pwd)"
dir=$root/target/parity/$name
mkdir -p "$dir"
"$@" 2>&1 | tee "$dir/log.txt"
status=${PIPESTATUS[0]}
reports=()
if [[ -n $glob ]]; then
  for f in $root/$glob; do [[ -f $f ]] && cp "$f" "$dir/" && reports+=("$f"); done
fi
excerpt() { # <title>, the text on stdin
  printf '<details open><summary>%s</summary>\n\n```\n' "$1"; cat; printf '```\n</details>\n\n'
}
{
  if ((status == 0)); then
    echo "- \`$name\`: passed"
  else
    echo "### \`$name\` failed (exit $status)"
    tail -n 25 "$dir/log.txt" | excerpt "output (last lines)"
    for f in "${reports[@]}"; do head -n 25 "$f" | excerpt "${f#"$root"/} (first lines)"; done
  fi
} >> "${GITHUB_STEP_SUMMARY:-/dev/null}"
exit "$status"
