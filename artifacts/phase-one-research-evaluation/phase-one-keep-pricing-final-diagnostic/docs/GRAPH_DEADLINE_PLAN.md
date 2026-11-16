# 彩色中の時計確認を間引く

main35のperf547sampleではclock_gettime系が約32%、主に彩色の頂点選択ごとのInstant::nowだった。彩色中は256選択ごとに確認し、節点入口と分岐の確認は従来通りに保つ。途中の彩色を上界へ使わない規則は変えない。選択数と時計確認数を結果へ保存する。

旧版は不変snapshot7831746b（experiments/binary-graph-snapshot-20261115.json）をcontrolにする。新旧を同CPUで交互に固定200万節点・十分な時間枠で比較し、status/primal/dual/全変数/根彩色/順列/根分岐完了数が同じことを確認する。全列挙/偽証明/期限0/小さい期限/元MPS独立検査も行い、時間超過を隠さない。

追加のopen入力sorrell7はdata/raw/exploratoryに別保存し、hashをexperiments/sorrell7-input-20261115.jsonへ記録する。全240 benchmarkとは分け、既知値をコード・初期解・停止条件へ渡さず汎用の経路だけで探索する。
