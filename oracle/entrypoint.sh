#!/bin/sh
# 결정론 오라클 실행: noto 코퍼스 생성 → rodf 빌드 → 스코어보드.
set -e

echo "== toolchain =="
soffice --version
pdftoppm -v 2>&1 | head -1
rustc --version

echo "== build rodf =="
cargo build -q -p rodf-cli

echo "== generate noto corpus =="
python3 tools/gen_corpus.py --font-set noto --out /tmp/corpus-noto

echo "== scoreboard =="
cd tools
python3 corpus.py --corpus-dir /tmp/corpus-noto \
    --out ../docs/scoreboard-ci.md --json ../docs/scoreboard-ci.json
