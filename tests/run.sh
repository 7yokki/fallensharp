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

$F build tests/fpp/arrays.fpp -o "$TMP/arrays" >/dev/null
out=$("$TMP/arrays")
[ "$out" = $'Selam
ok' ] || { echo "arrays FAIL: $out"; exit 1; }
echo "ok arrays"

$F build tests/fpp/bounds.fpp -o "$TMP/bounds" >/dev/null
set +e
out=$("$TMP/bounds" 2>&1)
code=$?
set -e
[ "$code" -eq 1 ] && echo "$out" | grep -q "sinir ihlali" || { echo "bounds FAIL"; exit 1; }
echo "ok bounds"

$F build tests/fpp/fatptr.fpp -o "$TMP/fatptr" >/dev/null
set +e
out=$("$TMP/fatptr" 2>&1)
code=$?
set -e
[ "$code" -eq 1 ] && echo "$out" | grep -q "sinir ihlali" || { echo "fatptr FAIL"; exit 1; }
echo "ok fatptr"

$F build tests/fpp/own_ok.fpp -o "$TMP/own_ok" --memory=own >/dev/null
out=$("$TMP/own_ok")
[ "$out" = "owned ok" ] || { echo "own_ok FAIL: $out"; exit 1; }
echo "ok own_ok"

if $F build tests/fpp/own_uaf.fpp -o "$TMP/own_uaf" --memory=own >/dev/null 2>"$TMP/own_uaf.err"; then
    echo "own_uaf FAIL: beklenmedik şekilde derlendi"; exit 1
fi
grep -q "taşınmış" "$TMP/own_uaf.err" || { echo "own_uaf FAIL: yanlış tanı"; cat "$TMP/own_uaf.err"; exit 1; }
echo "ok own_uaf"

if $F build tests/fpp/own_double_free.fpp -o "$TMP/own_df" --memory=own >/dev/null 2>"$TMP/own_df.err"; then
    echo "own_double_free FAIL: beklenmedik şekilde derlendi"; exit 1
fi
grep -q "double free" "$TMP/own_df.err" || { echo "own_double_free FAIL: yanlış tanı"; cat "$TMP/own_df.err"; exit 1; }
echo "ok own_double_free"

$F build tests/fpp/args8.fpp -o "$TMP/args8" >/dev/null
out=$("$TMP/args8")
[ "$out" = "nested ok" ] || { echo "args8 FAIL: $out"; exit 1; }
echo "ok args8"

$F build tests/fpp/floats.fpp -o "$TMP/floats" >/dev/null
out=$("$TMP/floats")
[ "$out" = $'6.500000\n5.500000\ncmp ok\n6.5' ] || { echo "floats FAIL: $out"; exit 1; }
echo "ok floats"

$F build tests/fpp/oop.fpp -o "$TMP/oop" >/dev/null
out=$("$TMP/oop")
[ "$out" = $'p=7\np2=17\nc=12' ] || { echo "oop FAIL: $out"; exit 1; }
echo "ok oop"

if $F build tests/fpp/oop_private.fpp -o "$TMP/oop_private" >/dev/null 2>"$TMP/oop_private.err"; then
    echo "oop_private FAIL: beklenmedik şekilde derlendi"; exit 1
fi
grep -q "özel" "$TMP/oop_private.err" || { echo "oop_private FAIL: yanlış tanı"; cat "$TMP/oop_private.err"; exit 1; }
echo "ok oop_private"

echo "TÜM FALLEN++ TESTLERİ GEÇTİ"
