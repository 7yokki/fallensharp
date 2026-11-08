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

$F build tests/fpp/safe_null.fpp -o "$TMP/safe_null" --memory=safe >/dev/null
set +e
out=$(printf '' | "$TMP/safe_null" 2>&1)
code=$?
set -e
[ "$code" -eq 1 ] && echo "$out" | grep -q "null pointer" || { echo "safe_null FAIL"; exit 1; }
echo "ok safe_null"

$F build tests/fpp/safe_div.fpp -o "$TMP/safe_div" --memory=safe >/dev/null
set +e
out=$("$TMP/safe_div" 2>&1)
code=$?
set -e
[ "$code" -eq 1 ] && echo "$out" | grep -q "sifira bolme" || { echo "safe_div FAIL"; exit 1; }
echo "ok safe_div"

echo "TÜM FALLEN++ TESTLERİ GEÇTİ"
