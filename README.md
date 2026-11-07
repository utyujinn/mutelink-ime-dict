# mutelink-ime-dict

The dictionary of Mutelink (a desktop app for talking in VRChat without one's own voice)'s offline kana-kanji conversion, and the tool that builds it.

- `kkc-dict/`: the binary dictionary format (reading and writing). Mutelink's conversion engine uses it as a git dependency.
- `builder/`: builds `mutelink-ime-dict.bin` from Mozc's open-source dictionary (`src/data/dictionary_oss`) and the extra words in `extra/`.
- `sources/`: which Mozc commit the data comes from, and a script that fetches those files and checks their SHA-256.
- `LICENSES/`, `NOTICE.md`: the licences of the data (Mozc: BSD-3-Clause; IPAdic: NAIST/ICOT terms; Okinawa dictionary: public domain). The dictionary is
  distributed with these texts, as the terms require.

## Build

```powershell
./sources/fetch.ps1                 # downloads the Mozc files into ./work/mozc and checks them
cargo run -p builder --release -- work/mozc extra/vrchat.tsv work/mutelink-ime-dict.bin "Mozc <commit>"
```

The same input gives the same bytes. Releases carry `mutelink-ime-dict-v1.zip` (the `.bin`, `LICENSES/` and `NOTICE.md`) and `SHA256SUMS`.

## Pinyin (Chinese)

The dictionary of the pinyin conversion, built from rime-essay (words and frequencies), OpenCC (traditional to simplified), pinyin-data and phrase-pinyin-data (readings):

- `pinyin-dict/`: the binary dictionary format (reading and writing). Mutelink's pinyin conversion uses it as a git dependency.
- `builder-zh/`: builds `mutelink-pinyin-dict.bin`.
- `sources/SOURCES-zh.md`, `sources/fetch-zh.ps1`: which commits the data comes from, and a script that fetches those files and checks their SHA-256. It also fetches the optional
  word-bigram language model (octagram, shipped unchanged).
- `NOTICE-zh.md`, `LICENSES/zh/`: the licences. The dictionary file contains LGPL-3.0 data (rime-essay), so it is distributed under the LGPL-3.0, as a separate file.

```powershell
./sources/fetch-zh.ps1              # downloads the pinyin data into ./work/zh and the octagram model into ./work/zh-gram, and checks them
cargo run -p builder-zh --release -- work/zh work/mutelink-pinyin-dict.bin "<meta text>"
python release/make_zip_zh.py work/mutelink-pinyin-dict.bin zh-v1 work/release-zh
```

Releases (tag `zh-dict-v1`) carry `mutelink-pinyin-zh-v1.zip` (the `.bin`, `LICENSES/zh/` and `NOTICE-zh.md`), the unchanged `zh-hans-t-essay-bgw-compact.gram`, and `SHA256SUMS`.

The code of this repository is MIT. The dictionary data is not: see `NOTICE.md` and `NOTICE-zh.md`.
