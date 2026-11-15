# mip-codex

Rustによる汎用MILPソルバーの開発。既存最適化ソルバーに中核を委譲しない。

現在は二段階密表単体法のLP参照実装のみ。MPS入力、MILP探索、独立検査器、大規模問題対応は未実装。SCIP/HiGHS/GLPKを上回る成績は未達成。

`cargo test` で数値コアの基本ケースを検証する。
`python3 scripts/fetch_miplib.py` で公式Benchmark Set v2を取得しSHA256と固定分割を保存する。

[開発計画](docs/PLAN.md) / [評価仕様](docs/BENCHMARK.md) / [作業記録](docs/JOURNAL.md)
