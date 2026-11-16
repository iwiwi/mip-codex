#!/usr/bin/env python3
"""公式benchmark v2を取得し、圧縮/展開双方のハッシュと固定分割を保存。"""
import hashlib
import json
from pathlib import Path
import urllib.request
import zipfile

root = Path(__file__).resolve().parents[1]
raw = root / 'data/raw'
raw.mkdir(parents=True, exist_ok=True)
meta = root / 'data'
for name in ('benchmark-v2.test', 'benchmark.zip'):
    path = raw / name
    if not path.exists():
        temp = path.with_suffix(path.suffix + '.part')
        urllib.request.urlretrieve('https://miplib.zib.de/downloads/' + name, temp)
        temp.rename(path)
with zipfile.ZipFile(raw / 'benchmark.zip') as archive:
    for info in archive.infolist():
        if Path(info.filename).is_absolute() or '..' in Path(info.filename).parts:
            raise ValueError('unsafe archive path')
    archive.extractall(raw / 'benchmark')
names = (raw / 'benchmark-v2.test').read_text().splitlines()
assert len(names) == len(set(names)) == 240
records = []
for name in names:
    matches = list((raw / 'benchmark').rglob(name))
    assert len(matches) == 1, (name, matches)
    path = matches[0]
    records.append(dict(name=name, path=str(path.relative_to(root)),
                        sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                        split='validation' if int(hashlib.sha256(name.encode()).hexdigest()[:8], 16) % 5 == 0 else 'development'))
manifest = dict(source='https://miplib.zib.de/downloads/benchmark.zip',
                archive_sha256=hashlib.sha256((raw/'benchmark.zip').read_bytes()).hexdigest(),
                list_sha256=hashlib.sha256((raw/'benchmark-v2.test').read_bytes()).hexdigest(), instances=records)
(meta/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
for split in ('development', 'validation'):
    (meta/(split+'.txt')).write_text(''.join(r['name']+'\n' for r in records if r['split']==split))
print('取得・ハッシュ完了:', len(records))
