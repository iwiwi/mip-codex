# MPS入力と解ファイル

`cargo build --release`

`target/release/mip-codex inspect model.mps.gz` は行列サイズ等をJSONで返す。
`target/release/mip-codex dump model.mps.gz` は元モデルを疎行形式のJSONで返す。
`target/release/mip-codex solve model.mps.gz 60 > solution.json` は時間制限60秒で実行する。
`.venv/bin/python scripts/validate_solution.py model.mps.gz solution.json` で元問題への独立検査を行う。実行可能解のない出力は検査対象外。

gzip/plain、MPSのROWS/COLUMNS/RHS/RANGES/BOUNDS、OBJSENSE/OBJNAME、整数marker、LO/UP/FX/FR/MI/PL/BV/LI/UIを実装。重複係数を加算。RANGESの符号、RHSの目的定数の反転、マーカーのみの整数変数の既定binary境界を解釈する。未知の拡張・複数ベクトル・不正数値・切断ファイルはエラー。

IBMの仕様から実装。既存ソルバーソースの移植なし。
https://www.ibm.com/docs/en/cofz/12.9.0?topic=standard-records-in-mps-format
https://www.ibm.com/docs/en/icos/22.1.1?topic=extensions-integer-variables-in-mps-files

元モデルの変数は解JSONのvaluesに名前付きで全て保存する。objectiveは元の目的方向・定数を含む。dual_boundがnullなら有限の双対境界がまだ得られていない。gapはabs(primal-dual)/max(1,abs(primal))。終了コード0はJSONの生成成功であり、最適解が得られたという意味ではない。statusと検査結果を参照する。

独立検査器の準備: `python3 -m venv .venv` と `.venv/bin/pip install -r scripts/requirements-comparison.txt`。Rust実行自体にPythonや比較ソルバーは不要。
