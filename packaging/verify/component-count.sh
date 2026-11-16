#!/usr/bin/env bash
# Shipped-component budget check (specs/004 research R13, FR-025, FR-034, SC-010).
#
# Counts unique packages (including the project's own four crates, exactly as defined in
# research R13) that the shipped application (`iv-app`, default features) includes across
# all target operating systems, excluding dev/build-only packages. Fails when the count
# exceeds the budget or when any network client is present in the shipped tree.
set -euo pipefail

BUDGET="${BUDGET:-411}"
FORBIDDEN=(reqwest hyper rustls tokio)

tree="$(cargo tree -p iv-app -e normal --target all --prefix none | sed 's/ (\*)//' | sort -u)"
count="$(printf '%s\n' "$tree" | grep -c . || true)"

echo "shipped_components=${count} budget=${BUDGET}"

status=0
for name in "${FORBIDDEN[@]}"; do
    if printf '%s\n' "$tree" | grep -qE "^${name} v"; then
        echo "FORBIDDEN in shipped tree: ${name}"
        status=1
    fi
done

if [ "$count" -gt "$BUDGET" ]; then
    echo "OVER BUDGET by $((count - BUDGET))"
    status=1
fi

exit "$status"
