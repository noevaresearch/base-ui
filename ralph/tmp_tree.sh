#!/usr/bin/env bash
cd /data/workspace/baseui
echo "=== 0.1.8 consumers ==="
cargo tree -i "reactive_graph@0.1.8" 2>&1 | head -14
echo "=== 0.2.14 consumers ==="
cargo tree -i "reactive_graph@0.2.14" 2>&1 | head -14
echo "=== leptos 0.7.8 deps (reactive_graph?) ==="
cargo tree -p leptos@0.7.8 2>&1 | grep -i "reactive" | head
