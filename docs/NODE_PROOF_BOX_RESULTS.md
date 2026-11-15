# 節点境界を使う証明用有限箱の試作（不採用）

2026-11-15。既存の元問題の区間演算による有限境界推論を、失敗した節点境界からも実行する試作。探索やLPには未接続。

`artifacts/infeasibility-box-initial/failure.json` のgen-ip002最初の失敗では、未設定上限は17個のまま、元の乗数/逆符号とも結果は `[None, 0]` で改善しなかった。従って既定実装には採用しない。169 Rustテスト成功、保存dumpを読む診断テスト成功（ログ artifacts/node-proof-box-tests.log、artifacts/node-proof-box-diagnostic.log）。元dumpの実行バイナリは別のnumerical-search-restart試作70c3329c、両実験オプションを無効にしたもの。receiptにソースとバイナリの照合を保存。

次の仮説：上限のない変数13,15,23のFarkas残差がゼロ付近で、単独乗数の微調整では全ての符号を保証できない。複数乗数の同時微調整候補を既存の区間証明器で検証する。小分母有理数比への丸めは、実際の小数係数から自然な比が得られず、この例への根拠がないので実装しない。
