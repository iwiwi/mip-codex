# 目的関数変更時の基底再利用（試作）

目的関数だけを変えるfeasibility pumpの準備として、独自Rust LPのwarm基底を新費用でも再利用する。係数行列・等式属性・スラック構造の一致は維持する。費用が異なる場合は基底値を再計算して全変数が境界内なら主単体法で最適化する。primal infeasibleならcold phase-Iへ戻す。費用が同じ従来ケースは双対単体法のまま。

親のArc構造は変更せず、新しい費用のsnapshotを作る。数値異常・非有界候補は既存cold fallbackと元単位の検査を通す。warm_cost_attempts/successesをLP統計とMILP集約へ追加する。外部の最適化処理は呼ばない。

初回は集約Statsの構築に新フィールドを加え忘れてコンパイル失敗した（warm-objective-initial-tests.log）。修正後、従来warm9試験が成功。さらに1000費用変更をcold revisedと別のdense単体法に照合し、元行/境界も検査する。費用+境界変更時のcold fallback、元基底の再利用、行列変更の棄却、単位1e-8/1e8と非有界rayも検査する。Rust136回帰が成功した。通常MILP探索は目的を変えないため、この機能だけによるMILP性能向上を主張しない。

次はrelease性能測定、全回帰と記録。その後、予算付きの根feasibility pumpを別変更として試す。元問題で有効な主解だけを採用し、局所探索のbound/infeasibleはグローバルへ持ち込まない。

## 部品測定

100モデル（24〜48変数、8〜20行、有限箱、等式を含む）で各10回費用変更、CPU0、HiGHS1thread。初回100回も含む1100解を原入力からFractionで境界/行/目的検査し、HiGHS1.16の最適値に照合、失敗0。1000変更でwarm成功1000、pivot37,035対cold53,057。時間合計はRust warm0.107108秒/cold0.146838秒、HiGHS warm0.144031秒。短い単発の部品測定なので全体速度の優位を示す値ではない。warm/cold/HiGHSの全返却点・入力・hashをartifacts/warm-objective-cost-seriesへ保存した。

再現: `cargo build --release --example reopt_lp`、`taskset -c 0 .venv/bin/python scripts/check_cost_warm.py`。実験ではCARGO_TARGET_DIRを親workspaceのtargetに指定し、専用worktreeのtarget symlinkから同じexampleを呼んだ。ソルバーMIP本体のreleaseは更新していない。通常MILPの4ソルバー比較は直前PR19の結果を参照し、本変更の成績と混同しない。
