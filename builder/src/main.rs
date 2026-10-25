//! Builds `mutelink-ime-dict.bin` from Mozc's `src/data/dictionary_oss` (fetched and checked by `sources/fetch.ps1`) and the extra words in `extra/`:
//!
//!   cargo run -p builder --release -- <mozc data dir> <extra tsv> <out file> <meta text>
//!
//! The same input gives the same bytes (kkc_dict::write puts everything in order), so a release can be rebuilt and compared.

use kkc_dict::{DictionaryData, EntryData, PosClass};
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        eprintln!("usage: builder <mozc data dir> <extra tsv> <out file> <meta text>");
        std::process::exit(2);
    }
    let (dir, extra, out, meta) = (&args[1], &args[2], &args[3], &args[4]);

    // id.def: "<id> <pos text>", ids in order from 0 (the beginning/end of a sentence).
    let id_def = fs::read_to_string(format!("{dir}/id.def")).expect("id.def");
    let mut pos_names: Vec<&str> = Vec::new();
    for (index, line) in id_def.lines().enumerate() {
        let (id, name) = line.split_once(' ').expect("an id and a part of speech");
        assert_eq!(id.parse::<usize>().expect("an id"), index, "id.def ids are in order");
        pos_names.push(name);
    }
    let pos_class: Vec<PosClass> = pos_names.iter().map(|name| classify(name)).collect();
    let common_noun = pos_names.iter().position(|name| name.starts_with("名詞,一般,")).expect("a common noun") as u16;

    // The connection costs: the first line is the count of parts of speech, then count^2 costs, one to a line.
    let connection = fs::read_to_string(format!("{dir}/connection_single_column.txt")).expect("connection_single_column.txt");
    let mut lines = connection.lines();
    let pos_count: usize = lines.next().expect("a count").trim().parse().expect("a count");
    assert_eq!(pos_count, pos_names.len(), "id.def and the connection matrix agree on the parts of speech");
    let matrix: Vec<i16> = lines.map(|line| line.trim().parse::<i16>().expect("a cost")).collect();
    assert_eq!(matrix.len(), pos_count * pos_count, "the matrix is pos count squared");

    // dictionary00..09: reading <TAB> left id <TAB> right id <TAB> cost <TAB> surface
    let mut entries: Vec<EntryData> = Vec::new();
    for i in 0..10 {
        let text = fs::read_to_string(format!("{dir}/dictionary0{i}.txt")).expect("a dictionary file");
        for line in text.lines() {
            let fields: Vec<&str> = line.split('\t').collect();
            assert!(fields.len() >= 5, "a dictionary line has five fields: {line}");
            entries.push(EntryData {
                reading: fields[0].to_string(),
                left: fields[1].parse().expect("a left id"),
                right: fields[2].parse().expect("a right id"),
                cost: fields[3].parse::<i32>().expect("a cost").clamp(0, u16::MAX as i32) as u16,
                surface: fields[4].to_string(),
            });
        }
    }
    let mozc_entries = entries.len();

    // The words the app adds (extra/*.tsv): reading <TAB> surface <TAB> cost (a lower cost wins over the same reading's other words).
    for line in fs::read_to_string(extra).expect("the extra words").lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        assert!(fields.len() == 3, "an extra word line has three fields: {line}");
        entries.push(EntryData { reading: fields[0].to_string(), left: common_noun, right: common_noun, cost: fields[2].parse().expect("a cost"), surface: fields[1].to_string() });
    }

    let data = DictionaryData { pos_class, matrix, entries, unknown_pos: common_noun, unknown_cost: 20000, meta: meta.clone() };
    let bytes = kkc_dict::write(&data);
    fs::write(out, &bytes).expect("the output file");
    println!("{} Mozc entries + {} extra, {} bytes -> {}", mozc_entries, data.entries.len() - mozc_entries, bytes.len(), out);
}

// How a part of speech behaves in a bunsetsu (see kkc_dict::PosClass): particles, auxiliary verbs, suffixes, words that cannot stand alone and the
// closing marks go with the word before; prefixes and opening brackets go with the word after.
fn classify(name: &str) -> PosClass {
    if name.starts_with("接頭詞") || name.starts_with("記号,括弧開") {
        PosClass::Prefix
    } else if name.starts_with("助詞") || name.starts_with("助動詞") || name.contains("接尾") || name.contains("非自立") || name.starts_with("記号") {
        PosClass::Attached
    } else {
        PosClass::Independent
    }
}
