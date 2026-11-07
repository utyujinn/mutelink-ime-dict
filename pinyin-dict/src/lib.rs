//! The binary dictionary of Mutelink's pinyin conversion: what the builder writes and the app reads.
//!
//! A key is a word's pinyin without tones, syllables joined (`ni` + `hao` is `nihao`; `ü` is `v`), so that a lookup needs no knowledge of where the syllables
//! end (`xian` is both `xi'an` and `xian`). A word has a frequency; the same key has several words, the most frequent first. One file, little endian:
//!
//! ```text
//! header   magic "MPYD", format version, key count, entry count, key bytes, word bytes, longest key (bytes), total frequency (low, high),
//!          meta length, tone bytes (u32 each), then the meta text (where the data came from)
//! key_offsets (key count + 1) x u32    into key bytes
//! key_first   (key count + 1) x u32    the first entry of each key (entries of one key are in a row, most frequent first)
//! key bytes
//! entry frequency   entry count x u32
//! word_offsets (entry count + 1) x u32    into word bytes
//! word bytes
//! tone_offsets (entry count + 1) x u32    into tone bytes
//! tone bytes   one byte for each character of each word (see `Entry::tones`)
//! ```
//!
//! Keys are sorted by their bytes, so a lookup is a binary search over `key_offsets`.

use std::ops::Range;

const MAGIC: &[u8; 4] = b"MPYD";
/// Raised whenever the layout above changes; the app asks the user to download the dictionary again when it meets one it does not know.
pub const FORMAT_VERSION: u32 = 2;
const HEADER_FIELDS: usize = 10;

/// An entry as the builder gets it.
#[derive(Clone, Debug)]
pub struct EntryData {
    pub key: String,
    pub word: String,
    pub freq: u32,
    /// For each character of the word, the tones it is read with under this key, as bits: bit 0 the neutral tone, bits 1 to 4 the four tones (a character with
    /// two readings of one key, 为 wéi and wèi, has two bits). One byte for each character of `word`.
    pub tones: Vec<u8>,
}

/// Everything the builder has gathered.
pub struct DictionaryData {
    pub entries: Vec<EntryData>,
    pub meta: String,
}

/// One word of a lookup.
#[derive(Clone, Copy, Debug)]
pub struct Entry<'a> {
    pub word: &'a str,
    pub freq: u32,
    /// The tone bits of each character (see `EntryData::tones`).
    pub tones: &'a [u8],
}

/// Writes the file. The entries are put in order (the same word under the same key given twice is kept once, with the higher frequency and the tones of both), so
/// that the same data always gives the same bytes.
pub fn write(data: &DictionaryData) -> Vec<u8> {
    let mut entries = data.entries.clone();
    entries.sort_by(|a, b| a.key.as_bytes().cmp(b.key.as_bytes()).then(b.freq.cmp(&a.freq)).then(a.word.cmp(&b.word)));
    entries.dedup_by(|b, a| {
        let same = a.key == b.key && a.word == b.word;
        if same {
            for (kept, other) in a.tones.iter_mut().zip(&b.tones) {
                *kept |= other;
            }
        }
        same
    });

    let mut key_offsets: Vec<u32> = Vec::new();
    let mut key_first: Vec<u32> = Vec::new();
    let mut key_bytes: Vec<u8> = Vec::new();
    let mut word_offsets: Vec<u32> = vec![0];
    let mut word_bytes: Vec<u8> = Vec::new();
    let mut tone_offsets: Vec<u32> = vec![0];
    let mut tone_bytes: Vec<u8> = Vec::new();
    let mut longest = 0usize;
    let mut total: u64 = 0;
    for (i, entry) in entries.iter().enumerate() {
        if i == 0 || entries[i - 1].key != entry.key {
            key_offsets.push(key_bytes.len() as u32);
            key_first.push(i as u32);
            key_bytes.extend_from_slice(entry.key.as_bytes());
            longest = longest.max(entry.key.len());
        }
        word_bytes.extend_from_slice(entry.word.as_bytes());
        word_offsets.push(word_bytes.len() as u32);
        assert_eq!(entry.tones.len(), entry.word.chars().count(), "a tone byte for each character of {}", entry.word);
        tone_bytes.extend_from_slice(&entry.tones);
        tone_offsets.push(tone_bytes.len() as u32);
        total += entry.freq as u64;
    }
    key_offsets.push(key_bytes.len() as u32);
    key_first.push(entries.len() as u32);
    let key_count = key_offsets.len() - 1;

    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(MAGIC);
    for field in [
        FORMAT_VERSION,
        key_count as u32,
        entries.len() as u32,
        key_bytes.len() as u32,
        word_bytes.len() as u32,
        longest as u32,
        total as u32,
        (total >> 32) as u32,
        data.meta.len() as u32,
        tone_bytes.len() as u32,
    ] {
        out.extend_from_slice(&field.to_le_bytes());
    }
    out.extend_from_slice(data.meta.as_bytes());
    for value in key_offsets.iter().chain(key_first.iter()) {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out.extend_from_slice(&key_bytes);
    for entry in &entries {
        out.extend_from_slice(&entry.freq.to_le_bytes());
    }
    for value in &word_offsets {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out.extend_from_slice(&word_bytes);
    for value in &tone_offsets {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out.extend_from_slice(&tone_bytes);
    out
}

/// A dictionary read from a file's bytes.
pub struct Dictionary {
    bytes: Vec<u8>,
    key_count: usize,
    entry_count: usize,
    longest_key: usize,
    total_freq: u64,
    meta: Range<usize>,
    key_offsets: usize,
    key_first: usize,
    key_bytes: usize,
    entry_freq: usize,
    word_offsets: usize,
    word_bytes: usize,
    tone_offsets: usize,
    tone_bytes: usize,
}

impl Dictionary {
    /// `Err` says why the bytes are not a dictionary this version can read (a different format version, a cut-off file).
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Dictionary, String> {
        if bytes.len() < 4 + HEADER_FIELDS * 4 || &bytes[..4] != MAGIC {
            return Err("this is not a Mutelink pinyin dictionary".to_string());
        }
        let field = |i: usize| u32::from_le_bytes(bytes[4 + i * 4..8 + i * 4].try_into().expect("four bytes")) as usize;
        if field(0) != FORMAT_VERSION as usize {
            return Err(format!("the dictionary is format version {}, this app reads {}", field(0), FORMAT_VERSION));
        }
        let (key_count, entry_count, key_len, word_len, longest_key) = (field(1), field(2), field(3), field(4), field(5));
        let total_freq = field(6) as u64 | (field(7) as u64) << 32;
        let meta_len = field(8);
        let tone_len = field(9);
        let meta_start = 4 + HEADER_FIELDS * 4;
        let key_offsets = meta_start + meta_len;
        let key_first = key_offsets + (key_count + 1) * 4;
        let key_bytes = key_first + (key_count + 1) * 4;
        let entry_freq = key_bytes + key_len;
        let word_offsets = entry_freq + entry_count * 4;
        let word_bytes = word_offsets + (entry_count + 1) * 4;
        let tone_offsets = word_bytes + word_len;
        let tone_bytes = tone_offsets + (entry_count + 1) * 4;
        if bytes.len() != tone_bytes + tone_len {
            return Err("the dictionary file is cut off or damaged".to_string());
        }
        Ok(Dictionary { bytes, key_count, entry_count, longest_key, total_freq, meta: meta_start..meta_start + meta_len, key_offsets, key_first, key_bytes, entry_freq, word_offsets, word_bytes, tone_offsets, tone_bytes })
    }

    fn u32_at(&self, base: usize, index: usize) -> usize {
        u32::from_le_bytes(self.bytes[base + index * 4..base + index * 4 + 4].try_into().expect("four bytes")) as usize
    }

    /// Where the data came from (the sources and their commits, the date).
    pub fn meta(&self) -> &str {
        std::str::from_utf8(&self.bytes[self.meta.clone()]).expect("the meta text is text")
    }
    pub fn entry_count(&self) -> usize {
        self.entry_count
    }
    pub fn key_count(&self) -> usize {
        self.key_count
    }
    /// The longest key in bytes: no word is looked up beyond it.
    pub fn longest_key(&self) -> usize {
        self.longest_key
    }
    /// The sum of the frequencies of all entries (to turn a frequency into a probability).
    pub fn total_frequency(&self) -> u64 {
        self.total_freq
    }

    fn key_at(&self, index: usize) -> &[u8] {
        &self.bytes[self.key_bytes + self.u32_at(self.key_offsets, index)..self.key_bytes + self.u32_at(self.key_offsets, index + 1)]
    }

    /// The entries of the words whose key is exactly `key`, the most frequent first (indices for `entry`); empty when there is none.
    pub fn lookup(&self, key: &str) -> Range<usize> {
        let target = key.as_bytes();
        let (mut low, mut high) = (0usize, self.key_count);
        while low < high {
            let middle = (low + high) / 2;
            match self.key_at(middle).cmp(target) {
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
                std::cmp::Ordering::Equal => return self.u32_at(self.key_first, middle)..self.u32_at(self.key_first, middle + 1),
            }
        }
        0..0
    }

    pub fn entry(&self, index: usize) -> Entry<'_> {
        let word = &self.bytes[self.word_bytes + self.u32_at(self.word_offsets, index)..self.word_bytes + self.u32_at(self.word_offsets, index + 1)];
        Entry {
            // The builder wrote these bytes from strings, so they are text.
            word: std::str::from_utf8(word).expect("the word bytes are text"),
            freq: self.u32_at(self.entry_freq, index) as u32,
            tones: &self.bytes[self.tone_bytes + self.u32_at(self.tone_offsets, index)..self.tone_bytes + self.u32_at(self.tone_offsets, index + 1)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> DictionaryData {
        // Tones: 你 is 3rd, 尼 2nd, 好 3rd (and 4th, hào), 你好 3rd and 3rd.
        let entry = |key: &str, freq: u32, word: &str, tones: &[u8]| EntryData { key: key.to_string(), word: word.to_string(), freq, tones: tones.to_vec() };
        DictionaryData {
            entries: vec![entry("nihao", 300, "你好", &[0b1000, 0b1000]), entry("ni", 900, "你", &[0b1000]), entry("ni", 100, "尼", &[0b100]), entry("hao", 800, "好", &[0b1000]), entry("ni", 900, "你", &[0b100])],
            meta: "test".to_string(),
        }
    }

    #[test]
    fn written_data_is_read_back() {
        let dictionary = Dictionary::from_bytes(write(&sample())).expect("a dictionary");
        assert_eq!(dictionary.meta(), "test");
        assert_eq!(dictionary.key_count(), 3);
        // The same word under the same key twice is kept once.
        assert_eq!(dictionary.entry_count(), 4);
        assert_eq!(dictionary.longest_key(), 5);
        assert_eq!(dictionary.total_frequency(), 300 + 900 + 100 + 800);
        let range = dictionary.lookup("ni");
        assert_eq!(range.len(), 2);
        // The most frequent first.
        assert_eq!(dictionary.entry(range.start).word, "你");
        assert_eq!(dictionary.entry(range.start + 1).word, "尼");
        assert_eq!(dictionary.entry(range.start + 1).freq, 100);
        assert_eq!(dictionary.lookup("n"), 0..0);
        // The tones are kept with the word, and the two readings of the same word under one key are one entry with both.
        assert_eq!(dictionary.entry(range.start).tones, &[0b1100]);
        assert_eq!(dictionary.entry(range.start + 1).tones, &[0b100]);
        assert_eq!(dictionary.entry(dictionary.lookup("nihao").start).tones, &[0b1000, 0b1000]);
    }

    #[test]
    fn the_same_data_gives_the_same_bytes() {
        let mut reversed = sample();
        reversed.entries.reverse();
        assert_eq!(write(&sample()), write(&reversed));
    }

    #[test]
    fn a_cut_off_file_is_refused() {
        let mut bytes = write(&sample());
        bytes.pop();
        assert!(Dictionary::from_bytes(bytes).is_err());
        assert!(Dictionary::from_bytes(b"nope".to_vec()).is_err());
    }
}
