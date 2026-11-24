#!/usr/bin/env bash
# Shipped-component budget check (specs/004 research R13, FR-025, FR-034, SC-010).
#
# Counts unique packages (including the project's own four crates; research R13) that the
# shipped application (`iv-app`, default features) includes across the three shipped desktop
# targets, excluding dev/build-only packages. Fails when the count
# exceeds the budget or when any network client is present in the shipped tree.
set -euo pipefail

BUDGET="${BUDGET:-356}"
FORBIDDEN=(reqwest hyper rustls tokio)

# Shipped desktop targets (wasm and Windows targets are excluded).
TARGETS=(aarch64-apple-darwin x86_64-apple-darwin x86_64-unknown-linux-gnu)
tree=""
for t in "${TARGETS[@]}"; do
    per_target="$(cargo tree --locked --color never -p iv-app -e normal --target "$t" --prefix none | sed 's/ (\*)//' | sort -u)"
    echo "target ${t}: $(printf '%s\n' "$per_target" | grep -c .)"
    tree="$(printf '%s\n%s\n' "$tree" "$per_target")"
done
tree="$(printf '%s\n' "$tree" | grep . | sort -u)"
if [ -n "${COMPONENT_LIST:-}" ]; then printf '%s\n' "$tree" >"$COMPONENT_LIST"; fi
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
