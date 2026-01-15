#!/bin/bash
# Triage binary builds

mkdir -p build_triage/{success,missing_deps,type_errors,other}

BINS=$(grep 'name = ' Cargo.toml | grep -A1 '\[\[bin\]\]' | grep 'name = ' | cut -d'"' -f2)

for bin in $BINS; do
    echo -n "Testing $bin... "
    if nix develop -c cargo build --bin "$bin" 2>&1 | tee "build_triage/logs/${bin}.log" | grep -q "Finished"; then
        echo "✅" 
        echo "$bin" >> build_triage/success/list.txt
    elif grep -q "use of unresolved" "build_triage/logs/${bin}.log"; then
        echo "📦 missing deps"
        echo "$bin" >> build_triage/missing_deps/list.txt
    elif grep -q "type annotations needed" "build_triage/logs/${bin}.log"; then
        echo "🔤 type errors"
        echo "$bin" >> build_triage/type_errors/list.txt
    else
        echo "❌ other"
        echo "$bin" >> build_triage/other/list.txt
    fi
done

echo ""
echo "📊 Triage Summary:"
echo "Success: $(wc -l < build_triage/success/list.txt 2>/dev/null || echo 0)"
echo "Missing deps: $(wc -l < build_triage/missing_deps/list.txt 2>/dev/null || echo 0)"
echo "Type errors: $(wc -l < build_triage/type_errors/list.txt 2>/dev/null || echo 0)"
echo "Other: $(wc -l < build_triage/other/list.txt 2>/dev/null || echo 0)"
