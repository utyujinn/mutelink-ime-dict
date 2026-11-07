# Where the pinyin data comes from

Everything is fetched at a pinned commit and checked by SHA-256 (`sources/fetch-zh.ps1`). To move to newer data, change the commit and the hash together.

| Data | Repository, commit | File | SHA-256 |
|---|---|---|---|
| Words and frequencies (traditional characters; LGPL-3.0) | rime/rime-essay `054920de4f54c9e5994276a96a4fc2a35cb51aa3` (2026-09-23) | essay.txt | 3e8512ebaa6657e35961b1f5762060717c647e3bf471d5e3b86e783b4e934ffb |
| | | AUTHORS | 4ab2f81451ff4819f2b0e346583b49cee37aec72a01f8ecd96be1711c4204b75 |
| | | LICENSE | da7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768 |
| Traditional to Taiwan's standard writing (Apache-2.0) | BYVoid/OpenCC `fae2a0a911c30764e669a7d8b0591cd8e852934a` (2026-10-29) | data/dictionary/TWVariants.txt | 245b94eb5842957e735dd44b7e7d4ff469a3643126cc8fa511adda5281e9cb86 |
| Traditional to simplified (Apache-2.0) | BYVoid/OpenCC `fae2a0a911c30764e669a7d8b0591cd8e852934a` (2026-10-29) | data/dictionary/TSCharacters.txt | 9ff46a7d30e5765375eb13d33f2b03a34d298913caf2b120380679f33ae1642d |
| | | data/dictionary/TSPhrases.txt | 1b1e60a3d50492bc088bcd4372152570b5f0ccbe2ded0b66c90b6b8eeb5f849f |
| | | LICENSE | b534e465949558eec2597b04f5092b5e161236a68dfbfd04d547592ac3964308 |
| Readings of characters (MIT) | mozillazg/pinyin-data `fe29ad32bdf352b31a02b9936e537f575432e9a6` (2026-11-01) | pinyin.txt | 2342a6f98c85dbc3c8d26aef31744876a36326c6466e51e9a0f16898c36c2011 |
| | | LICENSE | 9c048697be2502a16e8bcb282d5d465a07295b2def0ffb05a269c5d39dbe1586 |
| Readings of words (MIT) | mozillazg/phrase-pinyin-data `4afa3226ce53c9138b939e83c3bb294dbe0da511` (2026-11-01) | pinyin.txt | 5d4d86b1337bb2fca95bc5135564a821bfdb40597481ee3e3871bf52d4668622 |
| | | LICENSE | 89ac55df747e4776088c3e77531ef61b973a1a59dd8e6a4548a58996da9a4f70 |

The optional second package, the word-bigram language model (LGPL-3.0, shipped unchanged):

| Data | Source | File | SHA-256 |
|---|---|---|---|
| octagram, traditional, word bigrams (compact) | release 20260712 of lotem/rime-octagram-data | zh-hant-t-essay-bgw-compact.gram | 0488ebd6688f900a39200f2b794f2f99bcbf1e8fc27280ae4a2324b08b1559c1 |
| octagram, simplified, word bigrams (compact) | release 20260712 of lotem/rime-octagram-data | zh-hans-t-essay-bgw-compact.gram | d3cb2438c1fdcd6a855dd6ca8f5c1060a29273c6b64c2c2c69af67cd71b6aa7e |
| | `hans` branch `f8ce3b534733e489a8470a7c2adf5a154e8ea069` | LICENSE | da7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768 |

`phrase-pinyin-data` also has files made from other dictionaries (`large_pinyin.txt` and others); only its own `pinyin.txt` is used.
