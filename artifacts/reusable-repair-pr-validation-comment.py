from pathlib import Path
import json,subprocess,sys,os
os.sched_setaffinity(0,{43});root=Path.cwd();sys.path.insert(0,'scripts');from benchmark_common import require_solver_origin
info=require_solver_origin(root,root/'target/release/mip-codex');assert info['source_fingerprint']==info['library_source_fingerprint']=='8a9ce2724eeea001';url=next(x for x in Path('artifacts/reusable-repair-pr-create.log').read_text().splitlines()if x.startswith('https://github.com/iwiwi/mip-codex/pull/')); assert url.startswith('https://github.com/iwiwi/mip-codex/pull/');number=url.rsplit('/',1)[1];a=json.load(open('artifacts/reusable-repair-development-analysis.json'));assert a['completed_pairs']==188 and a['comparisons']['primal_improved']==2 and a['comparisons'].get('primal_worsened',0)==0;assert json.load(open('artifacts/reusable-repair-development-parallel-audit/summary.json'))['strict_valid']==128;assert json.load(open('artifacts/reusable-repair-default-direct-oracle/summary.json'))['gmp_valid']==400
text='''採用候補の測定・検証記録です。既定有効の最終FP8a9ce2724eeea001（core961e1b624f）を別凍結し、既定設定の188問題と全点・証明の再監査を実行中です。最終結果を待ってdraft解除・統合を判断します。

初期FP86f04a4091d4a4ae、同一凍結ELF flag0/1、開発188×各10秒・1thread・16GiB・CPU31..35・順序交互: 主解2改善/0悪化/186同等（map10 −16→−24、supportcase12 −9→−12）。下界3改善/2悪化/170同等/13比較不能、有限175件ずつ。両側183 time_limit /1 optimal /4 numerical_error。shift1幾何平均9.8605981064553→9.860597909263623、解決速度改善は確認できません。

全376出力を新規独立監査し合格、全128点GMP絶対1e−6・公式checker合格。実際の索引再利用は3問題。map10の10秒下界は悪化しましたが、独立CPU44の10/60秒比較8実行では全点・全元Fraction証明が合格し、60秒下界は両方−676.686083385692、主解は両時限−16→−24。map16715-04は主解−12と下界を保ち、10秒の修復約1.95→1.56秒です。機能非発動問題の下界変動は時刻打切りの影響を含みます。

CPU46×10秒のRust/HiGHS/SCIP主解はmap10 −24/−204/−205、map16715-04 −12/−22/−42。GLPKは主解なし。全6点GMP合格。最初のmap10 GLPKの比較専用driver欠落errorも保持し、primaryの既存driverへのsymlinkで両GLPKを再実行、両方time_limit/主解なしでした。総合優位・新記録は未達成です。

340 Rust成功/4 ignored。直接探索生成200モデル400点は全列挙最適値とGMPに合格、実際の索引再利用296回、改善592回。回帰1200出力・1028厳密下界証明・改ざん8224件拒否・全1200点GMPも合格。既定有効後も340/4と直接400点を再確認済み。主解専用の探索から下界・不能・最適性は移していません。目標拡大/失敗後再試行は診断で棄却し、全24点の証拠を別archiveへ保存しました。

初期測定archiveは14925ファイル528161418 bytes→85787205 bytes、6bundle、全payload SHA256検証済み、2ELFは元凍結場所に保持。採用後RHS archive17936 payloadを再検証してから、未追跡・非稼働・重複raw17075ファイル380934580 bytesのみ削除し、manifest/全bundle/凍結source/ELF/元root driver/logを保持しました。最終Rust240は未起動、holdout52は調整に使用していません。'''
u=json.loads(subprocess.check_output(['python3','scripts/ai_usage.py'],text=True));Path('artifacts/reusable-repair-pr-validation-ai-usage.json').write_text(json.dumps(u,ensure_ascii=False,indent=2));t=u['totals'];text+=f"\n\nAI使用量（直前ローカル最終累積、{u['sessions']} sessions）: total {t['total_tokens']:,}, input {t['input_tokens']:,}（cached {t['cached_input_tokens']:,}を含む）, output {t['output_tokens']:,}（reasoning {t['reasoning_output_tokens']:,}を含む）。内数の二重加算なし。\n";p=Path('artifacts/reusable-repair-pr-validation-comment.txt');p.write_text(text);r=subprocess.run(['gh','pr','comment',number,'--repo','iwiwi/mip-codex','--body-file',str(p)],capture_output=True,text=True);Path('artifacts/reusable-repair-pr-validation-comment.log').write_text(r.stdout+r.stderr);r.check_returncode();print(r.stdout)
