"""Packs the built dictionary with the licence texts into mutelink-ime-dict-<version>.zip and writes SHA256SUMS (same input, same bytes).

  python release/make_zip.py work/mutelink-ime-dict.bin v1 work/release
"""
import hashlib
import sys
import zipfile
from pathlib import Path

binary, version, out = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3])
root = Path(__file__).resolve().parent.parent
out.mkdir(parents=True, exist_ok=True)
files = [("mutelink-ime-dict.bin", binary), ("NOTICE.md", root / "NOTICE.md")] + [
    (f"LICENSES/{p.name}", p) for p in sorted((root / "LICENSES").iterdir())
]
archive = out / f"mutelink-ime-dict-{version}.zip"
with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    for name, path in files:
        info = zipfile.ZipInfo(name, date_time=(2026, 10, 25, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        info.external_attr = 0o644 << 16
        z.writestr(info, path.read_bytes(), compresslevel=9)
digest = hashlib.sha256(archive.read_bytes()).hexdigest()
(out / "SHA256SUMS").write_text(f"{digest}  {archive.name}\n")
print(digest, archive.stat().st_size, archive.name)
