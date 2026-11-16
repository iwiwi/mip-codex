# 次の診断仮説：列ノルムによるprimal価格付け補正

現在の改訂単体法は、符号を含むreduced costが最も負の非基底列を選ぶ。大きな根LPのpivot数と、HiGHS比較での時間差が残っている。列の長さを考慮した候補順位に変えることでpivot数が減るか、まず小さなRust差分で診断する。

候補の適格条件（reduced costの符号・tolerance・noise floor）、ratio test、原問題の点/下界/不能検査、許容誤差は変えない。順位だけをreduced/sqrt(1+sum(a^2))、またはreduced/(1+sum(abs(a)))で補正する。これは静的な列ノルムのヒューリスティックであり、動的なsteepest-edgeやDevexを実装したとは主張しない。退化時のBland順は維持する。

採用前に、凍結版と同じ開発側根問題、合成LP/native oracle、germanrr/neos-860300等の数値例、固定18問で調べる。別のpivot経路による数値失敗や主解悪化も保存し、根の速度だけで採用しない。nativeの点/重みをRustへ渡さない。最初は実験branchの定数で切り替え、効果がなければ新しい公開設定を増やさない。
