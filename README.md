# mutelink-ime-dict

The dictionary of [Mutelink](https://github.com/utyujinn/Mutelink)'s offline kana-kanji conversion, and the tool that builds it.

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

The code of this repository is MIT. The dictionary data is not: see `NOTICE.md`.
