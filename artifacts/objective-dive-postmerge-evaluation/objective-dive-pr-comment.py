from pathlib import Path
import json,subprocess
root=Path.cwd();head=json.loads((root/'artifacts/objective-dive-pr-rest.json').read_text())['head'];text=f'''最終候補FP344b84b997da532bをPR106上で検証しました。

- 開発188問×前後10秒、CPU3..7、1thread、各16GiB、同じ入力SHA。主解は1改善・1悪化・186同等です。exp-1は179115→137800、Binkarは6878.52→6888.550001。状態は両側171 time_limit / 11 resource_limit / 1 optimal / 5 numerical_errorで同じです。
- 有限下界は両側176問、比較は0改善・2悪化・174同等・12比較不能。下界の悪化も残し、解けた問題として数えていません。376出力の証明と122点の元MPS絶対検証がすべて有効です。
- 最終候補の60秒はexp-1が179115→131771、Binkarは両側6849.949995、ranは両側4459.0000002。6点6証明110カット有効。過去の60秒7問・swath独立再測定も保持し、追加探索呼出0で生じた差を改善効果としていません。
- 全276 Rust成功・4無視。同じ200元モデルをmin/maxと前後flagで計800実行し、全列挙/有理数頂点の最適値と一致、点・証明もすべて有効。800独立モデルではありません。

同CPU18・10秒・1thread・16GiBのnative比較:

| 問題 | Rust 主解/下界 | HiGHS | SCIP | GLPK |
| --- | --- | --- | --- | --- |
| exp-1-500-5-5 | 137800 / 33291.334578 | 65887 / 65887 | 65887 / 65887 | 71584 / 64220.229491 |

4点とRustの1証明32カットは独立有効。改善しても比較ソルバーには大きく劣ります。全体優位・新記録・未解決問題の攻略は主張しません。自作240問の本評価はまだ開始していません。

局所目的行、戻り分枝、改善成功時だけの再実行を追加します。1回0.25秒/512LP、最大4回、全体時間の5%・上限1秒。元問題で検査した点だけを返し、局所の不能/下界/基底を大域の枝刈りや証明に使いません。MIP_OBJECTIVE_LP_DIVE=0で新段階全体を止められます。

[結果・採用理由](https://github.com/iwiwi/mip-codex/blob/{head}/docs/OBJECTIVE_DIVE_RESULTS.md)。SHA検証済み圧縮記録は研究37,461ファイル（元1,154,717,324 bytes→160,353,271 bytes）と最終8,394ファイル（元200,743,228 bytes→27,477,195 bytes）。元ファイル・ELFはローカルに保持し、展開後payloadも全件SHA照合済みです。棄却した費用方向・浅い戻り方式、別の二値等式統合の負結果も保存しますが、そのコードは採用しません。PR106後の主作業木再現・本評価snapshot照合記録も含みます。
'''
usage=subprocess.check_output(['python3','scripts/ai_usage.py'],text=True);u=json.loads(usage);(root/'artifacts/objective-dive-pr-ai-usage.json').write_text(usage);t=u['totals'];text+=f"\nAI資源量（コメント直前のローカル最終累積、{u['sessions']} session）: total {t['total_tokens']:,}、input {t['input_tokens']:,}（cached {t['cached_input_tokens']:,}を含む）、output {t['output_tokens']:,}（reasoning {t['reasoning_output_tokens']:,}を含む）。内数は二重加算していません。\n";p=root/'artifacts/objective-dive-pr-comment.md';p.write_text(text);r=subprocess.run(['gh','pr','comment','107','--repo','iwiwi/mip-codex','--body-file',str(p)],capture_output=True,text=True);(root/'artifacts/objective-dive-pr-comment-result.json').write_text(json.dumps(dict(exit_code=r.returncode,stdout=r.stdout,stderr=r.stderr),indent=2)+'\n');print(r.stdout);print(r.stderr);assert r.returncode==0
