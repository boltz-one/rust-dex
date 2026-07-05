#!/usr/bin/env bash
#
# Publish the `boltz-*` (GPUI tree) crates to crates.io in dependency order.
#
# Single source of truth for publishing: used both by the GitHub Actions
# workflow (.github/workflows/publish.yml) and for manual/local fallback runs.
#
# Usage:
#   CARGO_REGISTRY_TOKEN=... ./scripts/publish-crates.sh            # real publish
#   DRY_RUN=1 ./scripts/publish-crates.sh                          # validate only, no upload
#   NO_VERIFY=0 CARGO_REGISTRY_TOKEN=... ./scripts/publish-crates.sh # publish with per-crate build verify
#
# Environment:
#   CARGO_REGISTRY_TOKEN  crates.io API token (required for real publish; passed to `cargo publish`)
#   DRY_RUN               "1" -> package + local checks only, never uploads (default "0")
#   NO_VERIFY             "1" -> pass --no-verify to cargo publish (default "1"; workspace is built separately)
#   ALLOW_DIRTY           "1" -> pass --allow-dirty (for local runs with uncommitted changes; default "0")
#
# Behaviour:
#   * Publishes in topological order (dependencies first).
#   * Skips any crate whose current version is already on crates.io (idempotent re-runs).
#   * After each real publish, waits for the version to appear on the sparse index
#     so the next dependent crate can resolve it.

set -euo pipefail

DRY_RUN="${DRY_RUN:-0}"
NO_VERIFY="${NO_VERIFY:-1}"
ALLOW_DIRTY="${ALLOW_DIRTY:-0}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Topological publish order (dependencies first). Package names as published on crates.io.
PACKAGES=(
  boltz-font-kit
  boltz-collections
  boltz-derive-refineable
  boltz-gpui-macros
  boltz-gpui-shared-string
  boltz-gpui-util
  boltz-util-macros
  boltz-util
  boltz-http-client
  boltz-media
  boltz-refineable
  boltz-scheduler
  boltz-logging
  boltz-tracing-facade-macros
  boltz-tracing-facade
  boltz-sum-tree
  boltz-gpui
  boltz-syntax-theme
  boltz-icons
  boltz-fonts-ibm-plex
  boltz-menu
  boltz-ui-macros
  boltz-gpui-wgpu
  boltz-gpui-windows
  boltz-theme
  boltz-gpui-linux
  boltz-gpui-macos
  boltz-component
  boltz-gpui-platform
  boltz-ui
  boltz-app
  gpui-probe
)

log() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[warn]\033[0m %s\n' "$*"; }
die() { printf '\033[1;31m[error]\033[0m %s\n' "$*" >&2; exit 1; }

# Resolve the version of a workspace package from cargo metadata.
pkg_version() {
  cargo metadata --format-version 1 --no-deps 2>/dev/null \
    | python3 -c "import sys,json;m=json.load(sys.stdin);print(next(p['version'] for p in m['packages'] if p['name']=='$1'))"
}

# Return 0 if $name@$version is already on crates.io.
already_published() {
  local name="$1" version="$2"
  python3 - "$name" "$version" <<'PY'
import sys, json, urllib.request, urllib.error
name, version = sys.argv[1], sys.argv[2]
n = name.lower()
if len(n) == 1: path = f"1/{n}"
elif len(n) == 2: path = f"2/{n}"
elif len(n) == 3: path = f"3/{n[0]}/{n}"
else: path = f"{n[:2]}/{n[2:4]}/{n}"
try:
    with urllib.request.urlopen(f"https://index.crates.io/{path}", timeout=20) as r:
        for line in r.read().decode().splitlines():
            if line.strip() and json.loads(line).get("vers") == version:
                sys.exit(0)  # found
    sys.exit(1)
except urllib.error.HTTPError as e:
    sys.exit(1 if e.code == 404 else 2)
except Exception:
    sys.exit(2)
PY
}

# Poll the sparse index until $name@$version becomes visible (max ~180s).
wait_for_index() {
  local name="$1" version="$2" i
  for i in $(seq 1 60); do
    if already_published "$name" "$version"; then
      log "  index has $name@$version"
      return 0
    fi
    sleep 3
  done
  warn "  timed out waiting for $name@$version on the index; continuing anyway"
}

publish_one() {
  local name="$1" version
  version="$(pkg_version "$name")"
  [ -n "$version" ] || die "cannot resolve version for $name"

  if already_published "$name" "$version"; then
    log "skip $name@$version (already on crates.io)"
    return 0
  fi

  local args=(publish -p "$name")
  [ "$NO_VERIFY" = "1" ] && args+=(--no-verify)
  [ "$ALLOW_DIRTY" = "1" ] && args+=(--allow-dirty)

  if [ "$DRY_RUN" = "1" ]; then
    log "dry-run: validate package $name@$version"
    local out
    if out="$(cargo "${args[@]}" --dry-run 2>&1)"; then
      echo "$out" | tail -n 3
      return 0
    fi
    # A dependent crate cannot be fully dry-run before its workspace deps are on
    # crates.io. Tolerate that specific case so the dry-run stays meaningful.
    if echo "$out" | grep -q 'no matching package named `boltz-'; then
      warn "  $name: manifest OK; full check deferred (workspace deps not yet on crates.io)"
      return 0
    fi
    echo "$out" >&2
    die "dry-run failed for $name"
  fi

  log "publish $name@$version"
  cargo "${args[@]}"
  wait_for_index "$name" "$version"
}

main() {
  if [ "$DRY_RUN" != "1" ] && [ -z "${CARGO_REGISTRY_TOKEN:-}" ]; then
    die "CARGO_REGISTRY_TOKEN is required for a real publish (set DRY_RUN=1 to validate only)"
  fi

  log "mode: $([ "$DRY_RUN" = 1 ] && echo DRY-RUN || echo PUBLISH) | no-verify=$NO_VERIFY | ${#PACKAGES[@]} crates"
  for p in "${PACKAGES[@]}"; do
    publish_one "$p"
  done
  log "done"
}

main "$@"
