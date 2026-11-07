# Notice (the pinyin data)

`mutelink-pinyin-dict.bin` is built from the data listed in `sources/SOURCES-zh.md`, at the commits recorded there. The licence texts are in `LICENSES/zh/`.

- **rime-essay** (`essay.txt`, the words and their weights) is licensed under the GNU Lesser General Public License v3.0 (`LICENSES/zh/rime-essay-LGPL-3.0.txt`).
  Its `AUTHORS` file says the data is refined from Chewing / 新酷音 (LGPL), OpenCC (Apache License 2.0), Android Pinyin IME (Apache License 2.0) and
  moedict.tw / 萌典 (CC0 1.0) (`LICENSES/zh/rime-essay-AUTHORS.txt`). **The dictionary file contains this data, so the dictionary file is distributed under the
  LGPL-3.0 as well**: it is a separate file that the app downloads, and the build is reproducible from `sources/` with `builder-zh`, so it can be replaced.
- **OpenCC** (`TSPhrases.txt`, `TSCharacters.txt`, used to turn the traditional characters into simplified ones) is licensed under the Apache License 2.0
  (`LICENSES/zh/OpenCC-Apache-2.0.txt`).
- **pinyin-data** and **phrase-pinyin-data** (the readings) are Copyright (c) 2016 mozillazg, licensed under the MIT License
  (`LICENSES/zh/pinyin-data-MIT.txt`, `LICENSES/zh/phrase-pinyin-data-MIT.txt`).
- **rime-octagram-data** (the optional language model `zh-hans-t-essay-bgw-compact.gram`) is licensed under the LGPL-3.0
  (`LICENSES/zh/rime-octagram-data-LGPL-3.0.txt`) and distributed unchanged. Its README says the file is refined from open internet corpora, made by an
  automatic process without human intervention, and that its maker takes no responsibility for its content.

The conversion uses these words and weights; it is provided without any warranty.
