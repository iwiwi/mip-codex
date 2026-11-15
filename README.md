# mip-codex

Rustによる汎用MILPソルバーの開発。既存最適化ソルバーに中核を委譲しない。

現在は二段階密表単体法と分枝限定による参照実装。MPS/gzip入力、有限/自由変数・整数変数・範囲制約に対応し、JSONで解と境界を返す。大規模問題用の疎LP・前処理・カットは未実装。SCIP/HiGHS/GLPKを上回る成績は未達成。

```
cargo test
cargo build --release
target/release/mip-codex solve model.mps.gz 60 > solution.json
.venv/bin/python scripts/validate_solution.py model.mps.gz solution.json
```

`python3 scripts/fetch_miplib.py` で公式Benchmark Set v2を取得しSHA256と固定分割を保存する。

[開発計画](docs/PLAN.md) / [評価仕様](docs/BENCHMARK.md) / [MPSと検査器](docs/MPS.md) / [作業記録](docs/JOURNAL.md)
