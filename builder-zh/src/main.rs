//! Builds `mutelink-pinyin-dict.bin` from the files that `sources/fetch-zh.ps1` puts in `work/zh` (and checks):
//!
//!   cargo run -p builder-zh --release -- <zh data dir> <out file> <meta text> [hant]
//!
//! With `hant` the words stay in the traditional characters of essay.txt, for the traditional-character keyboard, with the writing of Taiwan's standard (OpenCC's
//! `TWVariants.txt`: 裏 -> 裡, 着 -> 著, ...; the words themselves are not changed, so 網絡 stays 網絡); the readings of a word are looked
//! up through its simplified form (phrase-pinyin-data's words are simplified), and the characters that exist only in simplified writing are not added.
//!
//! - Words and frequencies: rime-essay's `essay.txt` (traditional characters), turned into simplified characters with OpenCC's `TSPhrases.txt` and
//!   `TSCharacters.txt` (the longest phrase first, then character by character).
//! - Readings: pinyin-data's `pinyin.txt` (every reading of a character, the first one the commonest) and phrase-pinyin-data's `pinyin.txt` (the readings
//!   of words). A word that is in no phrase file is read with the first reading of each of its characters.
//! - A key is the toneless pinyin of a word, syllables joined, `ü` as `v` (pinyin-dict's documentation says why). The tones the word is read with go into the
//!   entry (one byte for each character, the readings of the same key added up), so that a typed tone can narrow the candidates.
//!
//! The same input gives the same bytes (pinyin_dict::write puts everything in order), so a release can be rebuilt and compared.

use pinyin_dict::{DictionaryData, EntryData};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 && !(args.len() == 5 && args[4] == "hant") {
        eprintln!("usage: builder-zh <zh data dir> <out file> <meta text> [hant]");
        std::process::exit(2);
    }
    let (dir, out, meta) = (&args[1], &args[2], &args[3]);
    let hant = args.len() == 5;
    let read = |name: &str| fs::read_to_string(format!("{dir}/{name}")).unwrap_or_else(|e| panic!("{name}: {e}"));

    let phrases = opencc(&read("TSPhrases.txt"));
    let characters = opencc(&read("TSCharacters.txt"));
    let tw_variants = if hant { opencc(&read("TWVariants.txt")) } else { BTreeMap::new() };
    let longest_phrase = phrases.keys().map(|k| k.chars().count()).max().expect("OpenCC phrases");
    let to_simplified = |word: &str| -> String {
        let chars: Vec<char> = word.chars().collect();
        let mut out = String::new();
        let mut i = 0;
        while i < chars.len() {
            let mut converted = false;
            for length in (2..=longest_phrase.min(chars.len() - i)).rev() {
                let piece: String = chars[i..i + length].iter().collect();
                if let Some(simplified) = phrases.get(&piece) {
                    out.push_str(simplified);
                    i += length;
                    converted = true;
                    break;
                }
            }
            if !converted {
                match characters.get(&chars[i].to_string()) {
                    Some(simplified) => out.push_str(simplified),
                    None => out.push(chars[i]),
                }
                i += 1;
            }
        }
        out
    };

    // The characters that only the simplified writing has (what TSCharacters.txt turns characters into, and that it does not turn into anything itself).
    let simplified_only: BTreeSet<char> = characters.values().flat_map(|value| value.chars()).filter(|c| !characters.contains_key(&c.to_string())).collect();

    // essay.txt: word <TAB> weight; the same simplified word from several traditional ones adds up.
    let mut freq: BTreeMap<String, u64> = BTreeMap::new();
    for line in read("essay.txt").lines() {
        if line.starts_with('#') {
            continue;
        }
        let mut fields = line.split('\t');
        let (Some(word), Some(weight)) = (fields.next(), fields.next()) else { continue };
        let Ok(weight) = weight.trim().parse::<u64>() else { continue };
        let word = if hant { word.chars().map(|c| tw_variants.get(&c.to_string()).cloned().unwrap_or_else(|| c.to_string())).collect() } else { to_simplified(word) };
        if word.chars().all(is_ideograph) {
            *freq.entry(word).or_insert(0) += weight;
        }
    }

    // pinyin-data: "U+4E2D: zhōng,zhòng  # 中"
    let mut char_readings: BTreeMap<char, Vec<(String, u8)>> = BTreeMap::new();
    for line in read("char-pinyin.txt").lines() {
        if line.starts_with('#') {
            continue;
        }
        let Some((code, rest)) = line.split_once(": ") else { continue };
        let Some(hex) = code.strip_prefix("U+") else { continue };
        let ch = char::from_u32(u32::from_str_radix(hex, 16).expect("a code point")).expect("a character");
        let readings: Vec<(String, u8)> = rest.split('#').next().expect("a reading list").split(',').filter_map(|r| Some((toneless(r.trim())?, tone_bit(r.trim())))).collect();
        if !readings.is_empty() {
            char_readings.insert(ch, readings);
        }
    }

    // phrase-pinyin-data: "中国: zhōng guó", one reading a line (a word with several readings is on several lines).
    // The word -> its keys -> the tone bits of each character.
    let mut phrase_readings: BTreeMap<String, BTreeMap<String, Vec<u8>>> = BTreeMap::new();
    for line in read("phrase-pinyin.txt").lines() {
        if line.starts_with('#') {
            continue;
        }
        let Some((word, reading)) = line.split('#').next().expect("a line").split_once(':') else { continue };
        let syllables: Option<Vec<(String, u8)>> = reading.split_whitespace().map(|syllable| Some((toneless(syllable)?, tone_bit(syllable)))).collect();
        if let Some(syllables) = syllables {
            if syllables.len() == word.trim().chars().count() {
                let tones = phrase_readings.entry(word.trim().to_string()).or_default().entry(syllables.iter().map(|(key, _)| key.as_str()).collect::<String>()).or_insert_with(|| vec![0; syllables.len()]);
                for (tone, (_, bit)) in tones.iter_mut().zip(&syllables) {
                    *tone |= bit;
                }
            }
        }
    }

    let mut entries: Vec<EntryData> = Vec::new();
    let mut add = |key: String, word: &str, weight: u64, tones: Vec<u8>| {
        entries.push(EntryData { key, word: word.to_string(), freq: weight.clamp(1, u32::MAX as u64) as u32, tones });
    };
    for (word, weight) in &freq {
        let length = word.chars().count();
        if length == 1 {
            // A character: its commonest reading with the full weight, the others with a twentieth of it.
            let ch = word.chars().next().expect("a character");
            if let Some(readings) = char_readings.get(&ch) {
                for (i, (key, tones)) in by_key(readings).into_iter().enumerate() {
                    add(key, word, if i == 0 { *weight } else { (*weight / 20).max(1) }, vec![tones]);
                }
            }
        } else if let Some(keys) = phrase_readings.get(&if hant { to_simplified(word) } else { word.clone() }) {
            for (key, tones) in keys {
                add(key.clone(), word, *weight, tones.clone());
            }
        } else if let Some(first) = word.chars().map(|c| char_readings.get(&c).map(|r| r[0].clone())).collect::<Option<Vec<(String, u8)>>>() {
            add(first.iter().map(|(key, _)| key.as_str()).collect(), word, *weight, first.iter().map(|(_, bit)| *bit).collect());
        }
    }
    // A character that essay has no weight for still has to be typable.
    for (ch, readings) in &char_readings {
        if is_ideograph(*ch) && !freq.contains_key(&ch.to_string()) && !(hant && simplified_only.contains(ch)) {
            for (key, tones) in by_key(readings) {
                add(key, &ch.to_string(), 1, vec![tones]);
            }
        }
    }

    let count = entries.len();
    let bytes = pinyin_dict::write(&DictionaryData { entries, meta: meta.clone() });
    fs::write(out, &bytes).expect("the output file");
    println!("{} entries, {} bytes -> {}", count, bytes.len(), out);
}

// The readings of a character grouped by key, in the order of their first appearance (the first is the commonest), the tone bits of the readings of one key added up.
fn by_key(readings: &[(String, u8)]) -> Vec<(String, u8)> {
    let mut grouped: Vec<(String, u8)> = Vec::new();
    for (key, bit) in readings {
        match grouped.iter_mut().find(|(seen, _)| seen == key) {
            Some((_, tones)) => *tones |= bit,
            None => grouped.push((key.clone(), *bit)),
        }
    }
    grouped
}

// The tone of a syllable as a bit: bit 0 is the neutral tone (no mark), bits 1 to 4 the four tones (the marks, or the combining marks of "ê̄" and "ê̌").
fn tone_bit(syllable: &str) -> u8 {
    for c in syllable.chars() {
        let tone = match c {
            'ā' | 'ē' | 'ī' | 'ō' | 'ū' | 'ǖ' | '\u{304}' => 1,
            'á' | 'é' | 'í' | 'ó' | 'ú' | 'ǘ' | 'ń' | 'ḿ' | 'ế' | '\u{301}' => 2,
            'ǎ' | 'ě' | 'ǐ' | 'ǒ' | 'ǔ' | 'ǚ' | 'ň' | '\u{30c}' => 3,
            'à' | 'è' | 'ì' | 'ò' | 'ù' | 'ǜ' | 'ǹ' | 'ề' | '\u{300}' => 4,
            _ => continue,
        };
        return 1 << tone;
    }
    1
}

fn is_ideograph(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c)
}

// OpenCC's dictionary: "key <TAB> value1 value2 ..." (the first value is the one used), comments with '#'.
fn opencc(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let (key, values) = line.split_once('\t')?;
            Some((key.to_string(), values.split_whitespace().next()?.to_string()))
        })
        .collect()
}

// A syllable without its tone mark, `ü` as `v` (nǚ -> nv). Only the letters a-z are kept (this also drops the combining tone marks of "ê̄" and "ê̌"); `None` when
// nothing is left.
fn toneless(syllable: &str) -> Option<String> {
    let mut out = String::new();
    for c in syllable.chars() {
        out.push(match c {
            'ā' | 'á' | 'ǎ' | 'à' => 'a',
            'ē' | 'é' | 'ě' | 'è' | 'ê' | 'ế' | 'ề' => 'e',
            'ī' | 'í' | 'ǐ' | 'ì' => 'i',
            'ō' | 'ó' | 'ǒ' | 'ò' => 'o',
            'ū' | 'ú' | 'ǔ' | 'ù' => 'u',
            'ǖ' | 'ǘ' | 'ǚ' | 'ǜ' | 'ü' => 'v',
            'ń' | 'ň' | 'ǹ' => 'n',
            'ḿ' => 'm',
            'ɡ' => 'g',
            other => other,
        });
    }
    out.retain(|c| c.is_ascii_lowercase());
    (!out.is_empty()).then_some(out)
}
