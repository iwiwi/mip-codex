from pathlib import Path
import json,subprocess
root=Path.cwd();head=json.loads((root/'artifacts/objective-box-report-pr-rest.json').read_text())['head'];text=f'''開発188問の前後10秒比較を完了しました（1thread、各16GiB、前後を同じCPU21..25に固定、検証用52問は調整に不使用）。

- 有限下界は130→176問、新規46・消失0。主解は188問すべて同品質、状態分布も同じです。比較可能な下界は2改善・3悪化・125同等、58対は片側または両側不明です。新規下界は別にcoverageで数えています。
- 376出力の証明、122点の絶対元MPS検査はすべて有効です。解・境界・整数性は絶対1e-6、目的値は相対1e-8*(1+abs)、許容幅を緩めていません。
- 最終候補FP530f085a449593fdは既定有効で273 Rust試験成功・4無視。独立した小問題100個をmin/maxで200実行検証し、証明改ざん1,000件を拒否しました。初版のunderflow fixture失敗と既存4試験の報告機能隔離も記録し、旧assertを保持しました。
- 60秒5問は主解すべて同品質、10点10証明68カット有効。下界は3問同じ、glass-scは12.417576→12.412775と小さく悪化、seymour1は400.277486→400.285564と小さく改善しました。探索時間による差を結果から除外していません。

同CPU19・同制限のnative10秒比較（主解/下界、—は未報告）:

| 問題 | Rust | HiGHS | SCIP | GLPK |
| --- | --- | --- | --- | --- |
| atlanta-ip | — / 0 | 128.0183 / 81.6657 | — / 0.0015013 | — / — |
| map10 | — / -1675、resource limit | -204 / -586.1927 | -205 / -589.5354 | — / — |
| snp-02-004-104 | — / 103901250 | 587142258.7890 / 586783732.1326 | — / — | — / — |

Rustの3個のbox証明、比較側の4点はすべて独立有効です。多くが弱い箱下界です。この変更は探索の枝刈りや最適宣言を変えず、速度・最適解数で比較ソルバーを超えたという主張ではありません。全240問の自作ソルバー本評価はまだ開始していません。

[仮説・採否・詳細](https://github.com/iwiwi/mip-codex/blob/{head}/docs/OBJECTIVE_BOX_REPORT_HYPOTHESIS.md)。原記録はobjective-box-report-evaluation（8,007ファイル/193,647,026 bytes、ELF2個はローカル保持+SHA）とobjective-box-report-final-evaluation（2,177ファイル/40,677,563 bytes、ELF4個は同様）です。PR105後の再現・snapshot照合記録も含めています。
'''
usage=subprocess.check_output(['python3','scripts/ai_usage.py'],text=True);u=json.loads(usage);(root/'artifacts/objective-box-report-pr-ai-usage.json').write_text(usage);t=u['totals'];text+=f"\nAI資源量（このコメント直前のローカル最終累積ログ、{u['sessions']} session）: total {t['total_tokens']:,}、input {t['input_tokens']:,}（うちcached {t['cached_input_tokens']:,}）、output {t['output_tokens']:,}（うちreasoning {t['reasoning_output_tokens']:,}）。cachedはinputの内数、reasoningはoutputの内数です。\n";p=root/'artifacts/objective-box-report-pr-comment.md';p.write_text(text);r=subprocess.run(['gh','pr','comment','106','--repo','iwiwi/mip-codex','--body-file',str(p)],capture_output=True,text=True);(root/'artifacts/objective-box-report-pr-comment-result.json').write_text(json.dumps(dict(exit_code=r.returncode,stdout=r.stdout,stderr=r.stderr),indent=2)+'\n');print(r.stdout);print(r.stderr);assert r.returncode==0
