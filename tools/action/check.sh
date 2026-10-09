#!/bin/sh
# The `check` input of action.yml: `ktrs fmt --check` and/or `ktrs lint`, findings as GitHub annotations.
# Inputs arrive as KTRS_* variables (set in action.yml); exits with the last failing command's code.

# Not through PATH: a Windows install dir (`C:\...`) can't be a PATH entry of this shell.
ktrs=${KTRS_INSTALL_DIR:+$KTRS_INSTALL_DIR/}ktrs

set --
if [ "${KTRS_CHANGED_ONLY:-false}" = true ]; then
  base=${KTRS_BASE:-}
  if [ -z "$base" ]; then
    base=${KTRS_EVENT_BASE:-}
    # Events other than pull requests and pushes have no base, and a pushed new branch has none either (all zeros).
    case $base in *[!0]*) ;; *) base=$KTRS_DEFAULT_BASE ;; esac
    # A force push leaves the commit before it unreachable.
    if ! git cat-file -e "$base^{commit}" 2>/dev/null && git cat-file -e "$KTRS_DEFAULT_BASE^{commit}" 2>/dev/null; then
      base=$KTRS_DEFAULT_BASE
    fi
  fi
  set -- --changed-since "$base"
fi

status=0
for check in $(printf '%s' "${KTRS_CHECK:-}" | tr ',' ' '); do
  case $check in
    fmt) "$ktrs" fmt --check --reporter github "$@" ${KTRS_FMT_ARGS:-} || status=$? ;;
    lint) "$ktrs" lint --reporter github "$@" ${KTRS_LINT_ARGS:-} || status=$? ;;
    *)
      echo "::error title=ktrs::unknown check '$check' (expected fmt, lint, or both)"
      status=2
      ;;
  esac
done
exit $status
