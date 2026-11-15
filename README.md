# mip-codex

Rustで実装する汎用MILPソルバー。現在はMPS読込、二段階改訂単体法、分枝限定法を備える初期実装です。既存ソルバーを上回る性能はまだ達成していません。

```sh
cargo build --release
cargo test
python3 scripts/test_exhaustive.py
python3 -m venv .venv
.venv/bin/pip install highspy==1.16.0 pyscipopt==6.3.0 scipy==1.18.1
.venv/bin/python scripts/test_continuous.py
python3 scripts/fetch_miplib.py
target/release/mip-codex solve data/benchmark/pk1.mps.gz 10 > solution.json
.venv/bin/python scripts/check_solution.py data/benchmark/pk1.mps.gz solution.json
```

解はJSONで出力し、実行可能解・最適終了・実行不能・制限終了・数値失敗を区別します。`unbounded_relaxation` はMILP自体の非有界性の証明として扱いません。浮動小数点演算を使い、厳密な証明ログは未実装です。

比較専用GLPKドライバはシステムのGLPK開発パッケージからビルドします。Rust本体は既存のLP/MIPソルバーを呼び出しません。

```sh
mkdir -p tools
cc -O2 -Wall -Wextra scripts/glpk_driver.c -lglpk -lm -o tools/glpk_driver
.venv/bin/python scripts/freeze_environment.py
.venv/bin/python scripts/benchmark.py --seconds 10 --count 3 --output logs/my-run
```

評価条件は [docs/EVALUATION.md](docs/EVALUATION.md)、継続計画は [docs/PLAN.md](docs/PLAN.md)、再開時の状態は [docs/CURRENT_WORK.md](docs/CURRENT_WORK.md) を参照してください。データと生ログはローカルに保持し、ハッシュと実験結果をリポジトリに保存します。

Rust本体とPythonスクリプトはApache-2.0。比較専用の `scripts/glpk_driver.c` はGPL-3.0-or-laterです。Rust本体のfaerはMITライセンスの汎用線形代数ライブラリで、最適化アルゴリズムの委譲には使用しません。
