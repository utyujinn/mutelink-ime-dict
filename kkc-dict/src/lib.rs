//! The binary dictionary of Mutelink's kana-kanji conversion: what the builder writes and the app reads.
//!
//! The data is Mozc's open-source dictionary (the words with the readings, the part-of-speech ids and the costs) and its connection-cost matrix, laid
//! out so that a reading is found by a binary search in the file as it was read, without building a table at start. One file, little endian:
//!
//! ```text
//! header   magic "MKKC", format version, pos count, reading count, entry count, reading bytes, surface bytes, longest reading (chars),
//!          unknown pos, unknown cost, meta length (u32 each), then the meta text (where the data came from)
//! pos class      pos count x u8        (PosClass)
//! matrix         pos count^2 x i16     (cost of the pos `right` followed by the pos `left`: matrix[right * pos count + left])
//! reading_offsets (reading count + 1) x u32   into reading bytes
//! reading_first   (reading count + 1) x u32   the first entry of each reading (entries of one reading are in a row, cheapest first)
//! reading bytes
//! entry left / right / cost    entry count x u16 each
//! surface_offsets (entry count + 1) x u32     into surface bytes
//! surface bytes
//! ```
//!
//! Readings are sorted by their bytes, so a lookup is a binary search over `reading_offsets`.

use std::ops::Range;

const MAGIC: &[u8; 4] = b"MKKC";
/// Raised whenever the layout above changes; the app asks the user to download the dictionary again when it meets one it does not know.
pub const FORMAT_VERSION: u32 = 1;
const HEADER_FIELDS: usize = 10;

/// What a part of speech does in a bunsetsu (a phrase of the sentence the conversion shows as one unit).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum PosClass {
    /// A word that can begin a bunsetsu (nouns, verbs, adjectives...).
    Independent = 0,
    /// A word that goes with the one before it (particles, auxiliary verbs, suffixes, closing marks).
    Attached = 1,
    /// A word that goes with the one after it (prefixes, opening marks).
    Prefix = 2,
}

impl PosClass {
    fn from_u8(value: u8) -> PosClass {
        match value {
            1 => PosClass::Attached,
            2 => PosClass::Prefix,
            _ => PosClass::Independent,
        }
    }
}

/// An entry as the builder gets it.
#[derive(Clone, Debug)]
pub struct EntryData {
    pub reading: String,
    pub left: u16,
    pub right: u16,
    pub cost: u16,
    pub surface: String,
}

/// Everything the builder has gathered.
pub struct DictionaryData {
    pub pos_class: Vec<PosClass>,
    /// `pos_count^2` costs, `matrix[right * pos_count + left]`.
    pub matrix: Vec<i16>,
    pub entries: Vec<EntryData>,
    /// The part of speech and cost of a character that is in no word.
    pub unknown_pos: u16,
    pub unknown_cost: u16,
    pub meta: String,
}

/// One word of a lookup.
#[derive(Clone, Copy, Debug)]
pub struct Entry<'a> {
    pub left: u16,
    pub right: u16,
    pub cost: u16,
    pub surface: &'a str,
}

/// Writes the file. The entries are put in order (and the same word given twice is kept once, with the lower cost), so that the same data always gives
/// the same bytes.
pub fn write(data: &DictionaryData) -> Vec<u8> {
    let pos_count = data.pos_class.len();
    assert_eq!(data.matrix.len(), pos_count * pos_count, "the matrix is not pos count squared");
    let mut entries = data.entries.clone();
    entries.sort_by(|a, b| a.reading.as_bytes().cmp(b.reading.as_bytes()).then(a.cost.cmp(&b.cost)).then(a.left.cmp(&b.left)).then(a.surface.cmp(&b.surface)));
    entries.dedup_by(|b, a| a.reading == b.reading && a.left == b.left && a.right == b.right && a.surface == b.surface);

    let mut reading_offsets: Vec<u32> = Vec::new();
    let mut reading_first: Vec<u32> = Vec::new();
    let mut reading_bytes: Vec<u8> = Vec::new();
    let mut surface_offsets: Vec<u32> = vec![0];
    let mut surface_bytes: Vec<u8> = Vec::new();
    let mut longest = 0usize;
    for (i, entry) in entries.iter().enumerate() {
        if i == 0 || entries[i - 1].reading != entry.reading {
            reading_offsets.push(reading_bytes.len() as u32);
            reading_first.push(i as u32);
            reading_bytes.extend_from_slice(entry.reading.as_bytes());
            longest = longest.max(entry.reading.chars().count());
        }
        surface_bytes.extend_from_slice(entry.surface.as_bytes());
        surface_offsets.push(surface_bytes.len() as u32);
    }
    reading_offsets.push(reading_bytes.len() as u32);
    reading_first.push(entries.len() as u32);
    let reading_count = reading_offsets.len() - 1;

    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(MAGIC);
    for field in [
        FORMAT_VERSION,
        pos_count as u32,
        reading_count as u32,
        entries.len() as u32,
        reading_bytes.len() as u32,
        surface_bytes.len() as u32,
        longest as u32,
        data.unknown_pos as u32,
        data.unknown_cost as u32,
        data.meta.len() as u32,
    ] {
        out.extend_from_slice(&field.to_le_bytes());
    }
    out.extend_from_slice(data.meta.as_bytes());
    out.extend(data.pos_class.iter().map(|class| *class as u8));
    for value in &data.matrix {
        out.extend_from_slice(&value.to_le_bytes());
    }
    for value in reading_offsets.iter().chain(reading_first.iter()) {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out.extend_from_slice(&reading_bytes);
    for column in [|e: &EntryData| e.left, |e: &EntryData| e.right, |e: &EntryData| e.cost] {
        for entry in &entries {
            out.extend_from_slice(&column(entry).to_le_bytes());
        }
    }
    for value in &surface_offsets {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out.extend_from_slice(&surface_bytes);
    out
}

/// A dictionary read from a file's bytes.
pub struct Dictionary {
    bytes: Vec<u8>,
    pos_count: usize,
    reading_count: usize,
    entry_count: usize,
    longest_reading: usize,
    unknown_pos: u16,
    unknown_cost: u16,
    meta: Range<usize>,
    pos_class: usize,
    matrix: usize,
    reading_offsets: usize,
    reading_first: usize,
    reading_bytes: usize,
    entry_left: usize,
    entry_right: usize,
    entry_cost: usize,
    surface_offsets: usize,
    surface_bytes: usize,
}

impl Dictionary {
    /// `Err` says why the bytes are not a dictionary this version can read (a different format version, a cut-off file).
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Dictionary, String> {
        if bytes.len() < 4 + HEADER_FIELDS * 4 || &bytes[..4] != MAGIC {
            return Err("this is not a Mutelink kana-kanji dictionary".to_string());
        }
        let field = |i: usize| u32::from_le_bytes(bytes[4 + i * 4..8 + i * 4].try_into().expect("four bytes")) as usize;
        if field(0) != FORMAT_VERSION as usize {
            return Err(format!("the dictionary is format version {}, this app reads {}", field(0), FORMAT_VERSION));
        }
        let (pos_count, reading_count, entry_count, reading_len, surface_len, longest_reading) = (field(1), field(2), field(3), field(4), field(5), field(6));
        let (unknown_pos, unknown_cost, meta_len) = (field(7) as u16, field(8) as u16, field(9));
        let meta_start = 4 + HEADER_FIELDS * 4;
        let pos_class = meta_start + meta_len;
        let matrix = pos_class + pos_count;
        let reading_offsets = matrix + pos_count * pos_count * 2;
        let reading_first = reading_offsets + (reading_count + 1) * 4;
        let reading_bytes = reading_first + (reading_count + 1) * 4;
        let entry_left = reading_bytes + reading_len;
        let entry_right = entry_left + entry_count * 2;
        let entry_cost = entry_right + entry_count * 2;
        let surface_offsets = entry_cost + entry_count * 2;
        let surface_bytes = surface_offsets + (entry_count + 1) * 4;
        if bytes.len() != surface_bytes + surface_len {
            return Err("the dictionary file is cut off or damaged".to_string());
        }
        Ok(Dictionary {
            bytes,
            pos_count,
            reading_count,
            entry_count,
            longest_reading,
            unknown_pos,
            unknown_cost,
            meta: meta_start..meta_start + meta_len,
            pos_class,
            matrix,
            reading_offsets,
            reading_first,
            reading_bytes,
            entry_left,
            entry_right,
            entry_cost,
            surface_offsets,
            surface_bytes,
        })
    }

    fn u32_at(&self, base: usize, index: usize) -> usize {
        u32::from_le_bytes(self.bytes[base + index * 4..base + index * 4 + 4].try_into().expect("four bytes")) as usize
    }
    fn u16_at(&self, base: usize, index: usize) -> u16 {
        u16::from_le_bytes(self.bytes[base + index * 2..base + index * 2 + 2].try_into().expect("two bytes"))
    }

    /// Where the data came from (the Mozc commit, the date).
    pub fn meta(&self) -> &str {
        std::str::from_utf8(&self.bytes[self.meta.clone()]).expect("the meta text is text")
    }
    pub fn pos_count(&self) -> usize {
        self.pos_count
    }
    pub fn entry_count(&self) -> usize {
        self.entry_count
    }
    pub fn reading_count(&self) -> usize {
        self.reading_count
    }
    /// The longest reading in characters: no word is looked up beyond it.
    pub fn longest_reading(&self) -> usize {
        self.longest_reading
    }
    /// The part of speech and cost of a character that no word has.
    pub fn unknown(&self) -> (u16, u16) {
        (self.unknown_pos, self.unknown_cost)
    }
    pub fn pos_class(&self, pos: u16) -> PosClass {
        PosClass::from_u8(self.bytes[self.pos_class + pos as usize])
    }
    /// The cost of the part of speech `left` coming after `right` (pos 0 is the beginning and the end of the sentence).
    pub fn connection(&self, right: u16, left: u16) -> i32 {
        let index = right as usize * self.pos_count + left as usize;
        i16::from_le_bytes(self.bytes[self.matrix + index * 2..self.matrix + index * 2 + 2].try_into().expect("two bytes")) as i32
    }

    fn reading_at(&self, index: usize) -> &[u8] {
        &self.bytes[self.reading_bytes + self.u32_at(self.reading_offsets, index)..self.reading_bytes + self.u32_at(self.reading_offsets, index + 1)]
    }

    /// The entries of the words that are read exactly `reading`, cheapest first (indices for `entry`); empty when there is none.
    pub fn lookup(&self, reading: &str) -> Range<usize> {
        let target = reading.as_bytes();
        let (mut low, mut high) = (0usize, self.reading_count);
        while low < high {
            let middle = (low + high) / 2;
            match self.reading_at(middle).cmp(target) {
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
                std::cmp::Ordering::Equal => return self.u32_at(self.reading_first, middle)..self.u32_at(self.reading_first, middle + 1),
            }
        }
        0..0
    }

    pub fn entry(&self, index: usize) -> Entry<'_> {
        let surface = &self.bytes[self.surface_bytes + self.u32_at(self.surface_offsets, index)..self.surface_bytes + self.u32_at(self.surface_offsets, index + 1)];
        Entry {
            left: self.u16_at(self.entry_left, index),
            right: self.u16_at(self.entry_right, index),
            cost: self.u16_at(self.entry_cost, index),
            // The builder wrote these bytes from strings, so they are text.
            surface: std::str::from_utf8(surface).expect("the surface bytes are text"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> DictionaryData {
        let entry = |reading: &str, cost: u16, surface: &str| EntryData { reading: reading.to_string(), left: 1, right: 1, cost, surface: surface.to_string() };
        DictionaryData {
            pos_class: vec![PosClass::Independent, PosClass::Independent, PosClass::Attached],
            matrix: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
            entries: vec![entry("きょう", 3000, "今日"), entry("きょう", 2000, "京"), entry("あ", 1, "亜"), entry("きょう", 2000, "京")],
            unknown_pos: 1,
            unknown_cost: 9000,
            meta: "test".to_string(),
        }
    }

    #[test]
    fn written_data_is_read_back() {
        let dictionary = Dictionary::from_bytes(write(&sample())).expect("a dictionary");
        assert_eq!(dictionary.meta(), "test");
        assert_eq!(dictionary.reading_count(), 2);
        // The same word twice is kept once.
        assert_eq!(dictionary.entry_count(), 3);
        assert_eq!(dictionary.longest_reading(), 3);
        let range = dictionary.lookup("きょう");
        assert_eq!(range.len(), 2);
        // Cheapest first.
        assert_eq!(dictionary.entry(range.start).surface, "京");
        assert_eq!(dictionary.entry(range.start + 1).surface, "今日");
        assert_eq!(dictionary.lookup("きょ"), 0..0);
        assert_eq!(dictionary.connection(2, 1), 7);
        assert_eq!(dictionary.pos_class(2), PosClass::Attached);
        assert_eq!(dictionary.unknown(), (1, 9000));
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
