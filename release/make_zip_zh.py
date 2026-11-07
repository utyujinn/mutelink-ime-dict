"""Packs the pinyin dictionary with the licence texts into mutelink-pinyin-<version>.zip and writes SHA256SUMS (same input, same bytes).
The optional language model (the octagram .gram) is not packed: it is released as it is, next to the zip, with its licence.

  python release/make_zip_zh.py work/mutelink-pinyin-dict.bin zh-v2 work/release-zh-v2
  python release/make_zip_zh.py work/mutelink-pinyin-hant-dict.bin hant-v2 work/release-zh-hant-v2 hant   (the traditional-character dictionary and its model)
"""
import hashlib
import shutil
import sys
import zipfile
from pathlib import Path

binary, version, out = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3])
script = "hant" if len(sys.argv) > 4 and sys.argv[4] == "hant" else "hans"
root = Path(__file__).resolve().parent.parent
out.mkdir(parents=True, exist_ok=True)
files = [("mutelink-pinyin-dict-v2.bin", binary), ("NOTICE-zh.md", root / "NOTICE-zh.md")] + [
    (f"LICENSES/zh/{p.name}", p) for p in sorted((root / "LICENSES" / "zh").iterdir())
]
archive = out / f"mutelink-pinyin-{version}.zip"
with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    for name, path in files:
        info = zipfile.ZipInfo(name, date_time=(2026, 11, 7, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        info.external_attr = 0o644 << 16
        z.writestr(info, path.read_bytes(), compresslevel=9)
gram = root / "work" / "zh-gram" / f"zh-{script}-t-essay-bgw-compact.gram"
shutil.copyfile(gram, out / gram.name)
sums = []
for path in (archive, out / gram.name):
    sums.append(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}")
(out / "SHA256SUMS").write_text("\n".join(sums) + "\n")
print("\n".join(sums), archive.stat().st_size)
