#!/bin/sh
# Copyright 2026 Anud Inc.
# SPDX-License-Identifier: GPL-3.0-or-later

# Isonomic's acceptance check, run from the app itself. It needs no
# developer tools.
#
#   sh /Applications/Isonomic.app/Contents/Resources/acceptance-check
#
# The window drives itself twice, on a throwaway Capsule in the temporary
# folder: first run, a Chat turn, stop and continue; then a relaunch through
# a link. The Isonomic ID's keys stay in memory, so nothing is written to
# the keychain, and the Capsule is deleted when the check ends, pass or fail.
# The model is downloaded into that Capsule when none is given:
# ISONOMIC_CHECK_MODELS=<folder> uses model files already on this Mac.
# The check build also runs the full pass and keeps its screenshots, in a
# folder on the Desktop. Copy back everything the check prints.

set -u

here=$(cd "$(dirname "$0")" && pwd)
app=$(cd "$here/../.." && pwd)
window="${ISONOMIC_CHECK_WINDOW:-$app/Contents/MacOS/isonomic-desktop}"
prompt="${ISONOMIC_CHECK_PROMPT:-Plan one day in Lisbon}"
link="isonomic://chat/home"
memory_note="keys kept in memory: scratch Capsule"

if [ ! -x "$window" ]; then
    echo "acceptance check: FAIL (no window at $window)"
    exit 1
fi

full=0
[ -f "$app/Contents/Resources/check-build" ] && full=1

base="${TMPDIR:-/tmp}"
capsule=$(mktemp -d "${base%/}/isonomic-check.XXXXXX") || {
    echo "acceptance check: FAIL (no temporary folder)"
    exit 1
}
trap 'rm -rf "$capsule"' EXIT
trap 'exit 130' INT TERM

mkdir -p "$capsule/models"
# Marked before the engine first opens it: only a scratch Capsule may keep
# its keys in memory.
printf '{\n  "format": 1,\n  "scratch": true\n}\n' > "$capsule/capsule.json"

download=1
if [ -n "${ISONOMIC_CHECK_MODELS:-}" ]; then
    for file in "$ISONOMIC_CHECK_MODELS"/*; do
        [ -f "$file" ] || continue
        ln "$file" "$capsule/models/" 2>/dev/null || cp "$file" "$capsule/models/"
    done
    download=0
fi

# Who ran the check, without names: macOS, chip, memory and the app's build.
version=$(plutil -extract CFBundleShortVersionString raw "$app/Contents/Info.plist" 2>/dev/null)
build=$(plutil -extract CFBundleVersion raw "$app/Contents/Info.plist" 2>/dev/null)
memory=$(sysctl -n hw.memsize 2>/dev/null)
echo "Isonomic ${version:-?} (${build:-?}); macOS $(sw_vers -productVersion 2>/dev/null || uname -r);" \
    "$(uname -m); memory ${memory:-?} bytes; full pass $full"

# One run of the window in check mode. Prints its check lines; the status
# is the window's.
run() {
    mode=$1
    log="$capsule/self-check-$2.log"
    shift 2
    ISONOMIC_CAPSULE_DIR="$capsule" \
    ISONOMIC_SELF_CHECK="$mode" \
    ISONOMIC_KEY_STORE=memory \
    ISONOMIC_SELF_CHECK_FULL="$full" \
    ISONOMIC_SELF_CHECK_DOWNLOAD="$download" \
    ISONOMIC_SELF_CHECK_EXPECT="${expect:-}" \
        "$window" "$@" > /dev/null 2> "$log"
    status=$?
    grep '\[self-check\]' "$log"
    if [ "$status" -ne 0 ]; then
        last_lines
    fi
    return "$status"
}

# After a failed run: the engine's and the window's last log lines. Home
# folders print as ~.
last_lines() {
    for name in engine.stderr engine.log window.log; do
        file="$capsule/logs/$name"
        [ -s "$file" ] || continue
        echo "--- last lines of logs/$name"
        tail -n 20 "$file" | sed "s|$HOME|~|g"
    done
}

# Did the engine confirm, in its log, that each of its runs kept the keys
# in memory?
keys_in_memory() {
    count=$(grep -c "$memory_note" "$capsule/logs/engine.log" 2>/dev/null)
    if [ "${count:-0}" -ge "$1" ]; then
        echo "keys: memory (scratch Capsule), nothing written to the keychain"
        return 0
    fi
    echo "keys: FAIL, the engine did not confirm a memory key store"
    grep ISONOMIC_KEY_STORE "$capsule/logs/engine.log" 2>/dev/null | tail -n 1
    return 1
}

expect=""
prompt_ok=1
run "$prompt" prompt || prompt_ok=0
keys_in_memory 1 || prompt_ok=0
expect=$(grep -o ' state id=.*' "$capsule/self-check-prompt.log" | tail -n 1 | sed 's/^ state //')

link_ok=0
if [ "$prompt_ok" -eq 1 ]; then
    link_ok=1
    run link link "$link" || link_ok=0
    keys_in_memory 2 || link_ok=0
fi

if [ "$full" -eq 1 ] && [ -d "$capsule/screens" ]; then
    shots="$HOME/Desktop/Isonomic check screenshots $(date +%Y-%m-%d-%H%M%S)"
    cp -R "$capsule/screens" "$shots" && echo "screenshots: $shots"
fi

word() { [ "$1" -eq 1 ] && echo passed || echo failed; }
if [ "$prompt_ok" -eq 1 ] && [ "$link_ok" -eq 1 ]; then
    echo "acceptance check: PASS (prompt and link)"
    exit 0
fi
echo "acceptance check: FAIL (prompt $(word "$prompt_ok"), link $(word "$link_ok"))"
exit 1
