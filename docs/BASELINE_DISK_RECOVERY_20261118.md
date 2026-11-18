# Native baselineの外部ディスク障害・事前固定再測定

2026-11-18 06:33:06 UTC、開発側のdebug testリンクでルートディスクが満杯となり、native baseline管理プロセスもENOSPCで停止した。稼働GLPKは生存・OOM0。cargoと残compiler終端を確認して再生成可能debug cacheだけ削除し、空きを確保。06:43に凍結coordinatorを再開、既存719summaryのSHA、稼働case identity、06:30:21のsupervisor起動時刻を保持し、optimizer再起動なしを確認した。元のconfig39551aは不変。

この外部資源障害の影響を最終比較から曖昧にしないため、性能値に関係なく、保守的な06:32:00–06:34:00 UTCとsolver開始～summary完了時刻が重なる全jobを選んだ。該当はGLPK1件のみ。最初のpreflightはSCIPも残っていると仮定して2件を期待したが、実際の時刻は1件だったため副作用前にassert停止、初回script/logを保持して時刻照合結果に合わせた。候補解/境界/問題名からの選別はしない。

`baseline-v2-disk-recovery-20261118`に該当1件のcreate-only設定を用意した。元の全snapshotファイル/全library SHAを検査してコピーし、version/options・3600秒・16GiB・1thread・元CPU8..15の割当・swap0・pids512・validation600秒を維持。元240入力/manifest/instance_indexも維持し、jobだけを時間重複1件に限定。新しいid/config SHAと事前固定方針をreceiptに保存し、cgroup空・未起動を確認した。

元720件が終端し、元の全cgroup空・coordinator終端になってからだけ、別tagでこの1件を実行する。元結果と全エラーは削除/上書きしない。最終報告では外部障害の影響を疑う元1件と再測定結果を明示し、比較用の当該1件は事前固定した再測定結果を使用する。再測定が悪くても同じ方針で使用し、元の失敗・数値も付録へ残す。新しいnative測定の完了と予約CPU解放も、自作final240起動の必要条件に加える。holdout個別成績は開発判断に使わない。

証拠: `artifacts/large-capacity-baseline-recovery`、`large-capacity-baseline-disk-recovery-receipt.json`、`large-capacity-baseline-disk-recovery-config.json`、prepare/launch scriptsと初回preflight停止ログ。
