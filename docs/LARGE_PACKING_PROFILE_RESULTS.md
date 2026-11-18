# 大規模packing私的プロファイル・途中記録

本体main126/E5は変更していない。例示プログラムのprivate認識上限だけ65535へ拡張、ソース6ff5b0087a・4ファイルSHAと凍結ELFを保存した。example-only release build成功、通常CLIのSHA4c8e4148は前後同一。内部graph512MiB/weighted256MiB、外部16GiB・CPU47・1thread。

5000変数の独立pair生成問題では目的値−2500を0.584763秒で取得。全2500pairの厳密0/1制約および元MPSの独立GMP検査を通過、解析的最適値との一致は生成例の検査であり、一般問題の証明を返す機能ではない。

評価240問外の公式z26（17937変数）では、初期/延長20M反復・各60秒、singleが−1184（60.003612秒）、reseedが−1181（60.003763秒）。元MPSの独立GMP検査は両方valid。どちらも1ラウンドでdeadlineとなったため、この2本から再始動の有効性は比較できない。時間によって初期0.5秒探索の反復数も変わる。探索時間とは別にMPS読込・検査時間を記録している。双対境界・最適性・新記録・main性能の主張なし。

次の事前固定実験は、検証済みの最良自作点を共通seedとして各300秒。singleは100M反復上限、reseedは2Mごとに構築から再実行して乱数系列を変える。既知目的値や停止値は渡さず、全出力を元MPSのGMPで検査する。改善があれば本体機能への移植を別ソースで検討し、生成例・資源guard・開発188問の検査を行う。

空き領域確保では過去評価27032ファイル1,113,894,664バイトをprimaryの作業ツリーからのみsparse除外。全実体のSHA1 Git blob/SHA256と保持Git objectの型・サイズを照合し、baseline設定不変、untracked証拠と凍結ELF不変を確認した。証拠はGitに全保持。台帳large-packing-space-sparse.json。

公式[z26詳細](https://miplib.zib.de/instance_details_z26.html)を2026-11-18 UTCに参照し、掲載bestは−1198（2025-02-05登録）、openであることを確認。ここまでの−1184/−1181は新記録ではない。解のリンクは開かず、公式解・native解をoptimizerへ渡していない。公開bestは結果比較の文書だけに保持し探索の目標・停止には使わない。

300秒の共通seed継続は、single−1190（300.003840秒、1round、12,633,135反復）、reseed−1190（300.004131秒、7round）。元MPSのGMPは両方valid。全5候補点はすべてvalidで、新記録には未到達。容量移植候補はgraph-large-capacity別ブランチ（既定OFF）で本体の348Rust/4ignored、200小規模1200出力/7410tampers、40大規模解析例/GMP/証明と160tampersの検査を進め、開発188問を実行中。private結果から直接mainへ性能を帰属しない。
