#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release -q
F=target/release/fallenpp
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

$F build tests/fpp/hello.fpp -o "$TMP/hello" >/dev/null
out=$("$TMP/hello")
[ "$out" = $'Merhaba\ndogru' ] || { echo "hello FAIL: $out"; exit 1; }
echo "ok hello"

$F build tests/fpp/functions.fpp -o "$TMP/functions" >/dev/null
out=$("$TMP/functions")
[ "$out" = $'Hi Fallen\nok' ] || { echo "functions FAIL: $out"; exit 1; }
echo "ok functions"

$F build tests/fpp/noindent.fpp -o "$TMP/noindent" >/dev/null
out=$("$TMP/noindent")
[ "$out" = $'a\nb' ] || { echo "noindent FAIL: $out"; exit 1; }
echo "ok noindent"

$F build tests/fpp/io.fpp -o "$TMP/io" >/dev/null
out=$(printf 'Berat\n' | "$TMP/io")
[ "$out" = $'Ad: Selam Berat\nmerhaba' ] || { echo "io FAIL: $out"; exit 1; }
echo "ok io"

$F build tests/fpp/shell.fpp -o "$TMP/shell" >/dev/null
out=$(printf 'echo test\n' | "$TMP/shell")
echo "$out" | grep -q "test" || { echo "shell FAIL: $out"; exit 1; }
echo "ok shell"

echo "TÜM FALLEN++ TESTLERİ GEÇTİ"
