# Downloads the pinyin data listed in SOURCES-zh.md at the pinned commits into ../work/zh and checks their SHA-256 (stops at the first difference).
# Also fetches the octagram language model (the optional second package) into ../work/zh-gram.
$ErrorActionPreference = "Stop"
$files = @(
  @{ Name = "essay.txt";                    Repo = "rime/rime-essay";                Commit = "054920de4f54c9e5994276a96a4fc2a35cb51aa3"; Path = "essay.txt";                                Sha = "3e8512ebaa6657e35961b1f5762060717c647e3bf471d5e3b86e783b4e934ffb" },
  @{ Name = "essay-AUTHORS.txt";            Repo = "rime/rime-essay";                Commit = "054920de4f54c9e5994276a96a4fc2a35cb51aa3"; Path = "AUTHORS";                                  Sha = "4ab2f81451ff4819f2b0e346583b49cee37aec72a01f8ecd96be1711c4204b75" },
  @{ Name = "essay-LICENSE.txt";            Repo = "rime/rime-essay";                Commit = "054920de4f54c9e5994276a96a4fc2a35cb51aa3"; Path = "LICENSE";                                  Sha = "da7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768" },
  @{ Name = "TSCharacters.txt";             Repo = "BYVoid/OpenCC";                  Commit = "fae2a0a911c30764e669a7d8b0591cd8e852934a"; Path = "data/dictionary/TSCharacters.txt";         Sha = "9ff46a7d30e5765375eb13d33f2b03a34d298913caf2b120380679f33ae1642d" },
  @{ Name = "TSPhrases.txt";                Repo = "BYVoid/OpenCC";                  Commit = "fae2a0a911c30764e669a7d8b0591cd8e852934a"; Path = "data/dictionary/TSPhrases.txt";            Sha = "1b1e60a3d50492bc088bcd4372152570b5f0ccbe2ded0b66c90b6b8eeb5f849f" },
  @{ Name = "OpenCC-LICENSE.txt";           Repo = "BYVoid/OpenCC";                  Commit = "fae2a0a911c30764e669a7d8b0591cd8e852934a"; Path = "LICENSE";                                  Sha = "b534e465949558eec2597b04f5092b5e161236a68dfbfd04d547592ac3964308" },
  @{ Name = "char-pinyin.txt";              Repo = "mozillazg/pinyin-data";          Commit = "fe29ad32bdf352b31a02b9936e537f575432e9a6"; Path = "pinyin.txt";                               Sha = "2342a6f98c85dbc3c8d26aef31744876a36326c6466e51e9a0f16898c36c2011" },
  @{ Name = "pinyin-data-LICENSE.txt";      Repo = "mozillazg/pinyin-data";          Commit = "fe29ad32bdf352b31a02b9936e537f575432e9a6"; Path = "LICENSE";                                  Sha = "9c048697be2502a16e8bcb282d5d465a07295b2def0ffb05a269c5d39dbe1586" },
  @{ Name = "phrase-pinyin.txt";            Repo = "mozillazg/phrase-pinyin-data";   Commit = "4afa3226ce53c9138b939e83c3bb294dbe0da511"; Path = "pinyin.txt";                               Sha = "5d4d86b1337bb2fca95bc5135564a821bfdb40597481ee3e3871bf52d4668622" },
  @{ Name = "phrase-pinyin-data-LICENSE.txt"; Repo = "mozillazg/phrase-pinyin-data"; Commit = "4afa3226ce53c9138b939e83c3bb294dbe0da511"; Path = "LICENSE";                                 Sha = "89ac55df747e4776088c3e77531ef61b973a1a59dd8e6a4548a58996da9a4f70" }
)
$dir = Join-Path $PSScriptRoot "..\work\zh"
New-Item -ItemType Directory -Force $dir | Out-Null
foreach ($f in $files) {
  $path = Join-Path $dir $f.Name
  Invoke-WebRequest "https://raw.githubusercontent.com/$($f.Repo)/$($f.Commit)/$($f.Path)" -OutFile $path
  $actual = (Get-FileHash $path -Algorithm SHA256).Hash.ToLower()
  if ($actual -ne $f.Sha) { throw "$($f.Name) has SHA-256 $actual, expected $($f.Sha)" }
  Write-Host "ok $($f.Name)"
}

# The optional second package: the octagram language model (word bigrams), LGPL-3.0, shipped unchanged. The release asset of lotem/rime-octagram-data
# (tag 20260712); the licence is that of the `hans` branch at the commit below.
$gram = Join-Path $PSScriptRoot "..\work\zh-gram"
New-Item -ItemType Directory -Force $gram | Out-Null
$grams = @(
  @{ Name = "zh-hans-t-essay-bgw-compact.gram"; Url = "https://github.com/lotem/rime-octagram-data/releases/download/20260712/zh-hans-t-essay-bgw-compact.gram"; Sha = "d3cb2438c1fdcd6a855dd6ca8f5c1060a29273c6b64c2c2c69af67cd71b6aa7e" },
  @{ Name = "octagram-data-LICENSE.txt"; Url = "https://raw.githubusercontent.com/lotem/rime-octagram-data/f8ce3b534733e489a8470a7c2adf5a154e8ea069/LICENSE"; Sha = "da7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768" }
)
foreach ($f in $grams) {
  $path = Join-Path $gram $f.Name
  Invoke-WebRequest $f.Url -OutFile $path
  $actual = (Get-FileHash $path -Algorithm SHA256).Hash.ToLower()
  if ($actual -ne $f.Sha) { throw "$($f.Name) has SHA-256 $actual, expected $($f.Sha)" }
  Write-Host "ok $($f.Name)"
}
