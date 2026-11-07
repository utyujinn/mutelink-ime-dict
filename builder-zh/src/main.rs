//! Builds `mutelink-pinyin-dict.bin` from the files that `sources/fetch-zh.ps1` puts in `work/zh` (and checks):
//!
//!   cargo run -p builder-zh --release -- <zh data dir> <out file> <meta text>
//!
//! - Words and frequencies: rime-essay's `essay.txt` (traditional characters), turned into simplified characters with OpenCC's `TSPhrases.txt` and
//!   `TSCharacters.txt` (the longest phrase first, then character by character).
//! - Readings: pinyin-data's `pinyin.txt` (every reading of a character, the first one the commonest) and phrase-pinyin-data's `pinyin.txt` (the readings
//!   of words). A word that is in no phrase file is read with the first reading of each of its characters.
//! - A key is the toneless pinyin of a word, syllables joined, `ü` as `v` (pinyin-dict's documentation says why).
//!
//! The same input gives the same bytes (pinyin_dict::write puts everything in order), so a release can be rebuilt and compared.

use pinyin_dict::{DictionaryData, EntryData};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("usage: builder-zh <zh data dir> <out file> <meta text>");
        std::process::exit(2);
    }
    let (dir, out, meta) = (&args[1], &args[2], &args[3]);
    let read = |name: &str| fs::read_to_string(format!("{dir}/{name}")).unwrap_or_else(|e| panic!("{name}: {e}"));

    let phrases = opencc(&read("TSPhrases.txt"));
    let characters = opencc(&read("TSCharacters.txt"));
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

    // essay.txt: word <TAB> weight; the same simplified word from several traditional ones adds up.
    let mut freq: BTreeMap<String, u64> = BTreeMap::new();
    for line in read("essay.txt").lines() {
        if line.starts_with('#') {
            continue;
        }
        let mut fields = line.split('\t');
        let (Some(word), Some(weight)) = (fields.next(), fields.next()) else { continue };
        let Ok(weight) = weight.trim().parse::<u64>() else { continue };
        let word = to_simplified(word);
        if word.chars().all(is_ideograph) {
            *freq.entry(word).or_insert(0) += weight;
        }
    }

    // pinyin-data: "U+4E2D: zhōng,zhòng  # 中"
    let mut char_readings: BTreeMap<char, Vec<String>> = BTreeMap::new();
    for line in read("char-pinyin.txt").lines() {
        if line.starts_with('#') {
            continue;
        }
        let Some((code, rest)) = line.split_once(": ") else { continue };
        let Some(hex) = code.strip_prefix("U+") else { continue };
        let ch = char::from_u32(u32::from_str_radix(hex, 16).expect("a code point")).expect("a character");
        let readings: Vec<String> = rest.split('#').next().expect("a reading list").split(',').filter_map(|r| toneless(r.trim())).collect();
        if !readings.is_empty() {
            char_readings.insert(ch, readings);
        }
    }

    // phrase-pinyin-data: "中国: zhōng guó", one reading a line (a word with several readings is on several lines).
    let mut phrase_readings: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for line in read("phrase-pinyin.txt").lines() {
        if line.starts_with('#') {
            continue;
        }
        let Some((word, reading)) = line.split('#').next().expect("a line").split_once(':') else { continue };
        let syllables: Option<Vec<String>> = reading.split_whitespace().map(toneless).collect();
        if let Some(syllables) = syllables {
            if syllables.len() == word.trim().chars().count() {
                phrase_readings.entry(word.trim().to_string()).or_default().insert(syllables.concat());
            }
        }
    }

    let mut entries: Vec<EntryData> = Vec::new();
    let mut add = |key: String, word: &str, weight: u64| {
        entries.push(EntryData { key, word: word.to_string(), freq: weight.clamp(1, u32::MAX as u64) as u32 });
    };
    for (word, weight) in &freq {
        let length = word.chars().count();
        if length == 1 {
            // A character: its commonest reading with the full weight, the others with a twentieth of it.
            let ch = word.chars().next().expect("a character");
            if let Some(readings) = char_readings.get(&ch) {
                let mut seen = BTreeSet::new();
                for (i, reading) in readings.iter().enumerate() {
                    if seen.insert(reading) {
                        add(reading.clone(), word, if i == 0 { *weight } else { (*weight / 20).max(1) });
                    }
                }
            }
        } else if let Some(keys) = phrase_readings.get(word) {
            for key in keys {
                add(key.clone(), word, *weight);
            }
        } else if let Some(key) = word.chars().map(|c| char_readings.get(&c).map(|r| r[0].clone())).collect::<Option<Vec<String>>>() {
            add(key.concat(), word, *weight);
        }
    }
    // A character that essay has no weight for still has to be typable.
    for (ch, readings) in &char_readings {
        if is_ideograph(*ch) && !freq.contains_key(&ch.to_string()) {
            for reading in readings.iter().collect::<BTreeSet<_>>() {
                add(reading.clone(), &ch.to_string(), 1);
            }
        }
    }

    let count = entries.len();
    let bytes = pinyin_dict::write(&DictionaryData { entries, meta: meta.clone() });
    fs::write(out, &bytes).expect("the output file");
    println!("{} entries, {} bytes -> {}", count, bytes.len(), out);
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
