# MIPLIB 2017 v2 比較baseline v1

全240問×HiGHS/SCIP/GLPK=720ケース。3600秒・1thread・8枠CPU8..15・ケース合計16GiB。snapshotのソースcommitはconfig.json参照。起動2026-11-15 UTC。

完全な出力はリポジトリ内 `artifacts/runs/baseline-v1-20261115/` に保存 (gitignore)。ここには固定config、依存版、機械、保存時点の進捗、kernelによる生存/affinity確認をcommitする。progress/launch-evidenceは保存時点の記録であり現在の生存を保証しない。

```
.venv/bin/python scripts/benchmark.py status artifacts/runs/baseline-v1-20261115
```

再開は固定snapshotのbenchmark.py runを使用。詳細はdocs/EVALUATION_RUNNER.md。全720件の終了・検証・参照値監査・最終集計は未完了。
