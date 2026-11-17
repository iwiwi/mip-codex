from pathlib import Path
import json,subprocess
r=Path.cwd();audit=json.loads((r/'artifacts/resource-compact-final-primary-audit/summary.json').read_text());assert audit['checked']==audit['strict_valid']==audit['official_valid']==audit['cut_valid']==2
text='''main統合後の確認: main 5860b176f58f5e599be816f5eb4ca4b28736e608、通常/ライブラリのsource fingerprint 30859e006f1e0445。map10、map16715-04の60秒再実行は両方とも目的値0の実行可能点を保持し、2点とも元MPSのGMP絶対誤差検査と公式検査に合格、出力証明2件も合格しました。最適性や比較ソルバー超過は主張しません。

最終240問用のcreate-only snapshot resource-compact-final-20261117も作成済みです（config SHA256 5e6f837e312fe9bd94cbf7678d4715ed3158ea2ef9b334006a0dde378d965fbc、3600秒・1 thread・16GiB・CPU8..15）。ジョブは未起動です。

その後の独立した境界値調査で、mainにも以前から存在する微小な十進境界と変数shiftによる誤った最適性判定を4種類確認しました。Cold dualの有無の両方で再現するため、性能実験の採用を止め、元十進数から整数行・変数境界を厳密にceil/floorする安全性修正を別ブランチで検査しています。このsnapshotは修正版に置き換えるまで最終評価に使用しません。再現モデル・誤答の出力を保持しています。
'''
u=json.loads(subprocess.check_output(['python3','scripts/ai_usage.py'],text=True));(r/'artifacts/resource-compact-postmerge-ai-usage.json').write_text(json.dumps(u,ensure_ascii=False,indent=2));t=u['totals'];text+=f"\nAI資源量（コメント直前累積、{u['sessions']} sessions）: total {t['total_tokens']:,}, input {t['input_tokens']:,}（cached {t['cached_input_tokens']:,}を含む）, output {t['output_tokens']:,}（reasoning {t['reasoning_output_tokens']:,}を含む）。内数二重加算なし。\n";p=r/'artifacts/resource-compact-postmerge-comment.md';p.write_text(text);done=subprocess.run(['gh','pr','comment','116','--repo','iwiwi/mip-codex','--body-file',str(p)],capture_output=True,text=True);(r/'artifacts/resource-compact-postmerge-comment.log').write_text(done.stdout+done.stderr);assert done.returncode==0;print(done.stdout)
