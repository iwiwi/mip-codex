main統合後の確認: 4b9cf54aa7056e2b24557193db2467792ea805f6を主作業木で再buildし、通常/libraryのoriginとFPcdfc159e435d9194一致を確認。固定8例は正しい最適/不能、係数scale/純粋LP/変数境界等11例も期待通り（5撤回/4最適/2不能）。計15候補点が元MPSのGMP絶対検査に合格しました。bab2/bab6の60秒再測定は両方主解なし・下界−6011877.36200701/−4508908.021503733で同値、出力証明2件が独立有効です。

新しいcreate-only全240snapshotはinteger-bound-safety-final-20261117。config SHA256 08c9843112b030542e234fadbad3cfb2e8aa63b82d008af17169a31295fc1c7f、107 snapshotファイルと147共有libのSHAを照合済み。各3600秒・1thread・16GiB・CPU8..15。ジョブ未起動で、旧誤答版snapshotも起動していません。最終Rustは11月19日UTC目安の起動予定を維持します。性能の総合優位/新記録はありません。

AI資源量（コメント直前累積、1 sessions）: total 1,483,357,501, input 1,476,219,155（cached 1,465,556,352を含む）, output 7,138,346（reasoning 4,343,301を含む）。内数二重加算なし。
