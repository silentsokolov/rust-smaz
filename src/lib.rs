//! This crate implements the smaz algorithm for compressing very short strings.
//!
//! Smaz instead is not good for compressing general purpose data, but can
//! compress text by 40-50% in the average case (works better with English), and
//! is able to perform a bit of compression for HTML and urls as well. The
//! important point is that Smaz is able to compress even strings of two or
//! three bytes!
//!
//! See original [library by antirez](http://github.com/antirez/smaz) for information on smaz and the algorithm itself.
//!
//!
//! # Quick Start
//!
//! ```
//! extern crate smaz;
//!
//! use smaz::{compress,decompress};
//!
//! fn main() {
//!     let s = "my long string";
//!
//!     let compressed = compress(s.as_bytes());
//!     println!("bytes: {:?}", &compressed);
//!
//!     let decompressed = decompress(&compressed);
//!     if let Ok(v) = decompressed {
//!         println!("bytes: {:?}", &v);
//!     }
//! }
//! ```
//!
//!
//! ## Compression examples
//!
//! - `This is a small string` compressed by 50%
//! - `foobar` compressed by 34%
//! - `the end` compressed by 58%
//! - `not-a-g00d-Exampl333` *enlarged* by 15%
//! - `Smaz is a simple compression library` compressed by 39%
//! - `Nothing is more difficult, and therefore more precious, than to be able
//!   to decide` compressed by 49%
//! - `this is an example of what works very well with smaz` compressed by 49%
//! - `1000 numbers 2000 will 10 20 30 compress very little` compressed by 10%
//! - `and now a few italian sentences:` compressed by 41%
//! - `Nel mezzo del cammin di nostra vita, mi ritrovai in una selva oscura`
//!   compressed by 33%
//! - `Mi illumino di immenso` compressed by 37%
//! - `L'autore di questa libreria vive in Sicilia` compressed by 28%
//! - `try it against urls` compressed by 37%
//! - `http://google.com` compressed by 59%
//! - `http://programming.reddit.com` compressed by 52%

#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs
)]

#[macro_use]
extern crate lazy_static;

use std::error::Error;
use std::fmt;
use std::result;
use std::str;

/// Compression codebook, used for compression
pub static CODEBOOK: [&str; 254] = [
    " ", "the", "e", "t", "a", "of", "o", "and", "i", "n", "s", "e ", "r", " th", " t", "in", "he",
    "th", "h", "he ", "to", "\r\n", "l", "s ", "d", " a", "an", "er", "c", " o", "d ", "on", " of",
    "re", "of ", "t ", ", ", "is", "u", "at", "   ", "n ", "or", "which", "f", "m", "as", "it",
    "that", "\n", "was", "en", "  ", " w", "es", " an", " i", "\r", "f ", "g", "p", "nd", " s",
    "nd ", "ed ", "w", "ed", "http://", "for", "te", "ing", "y ", "The", " c", "ti", "r ", "his",
    "st", " in", "ar", "nt", ",", " to", "y", "ng", " h", "with", "le", "al", "to ", "b", "ou",
    "be", "were", " b", "se", "o ", "ent", "ha", "ng ", "their", "\"", "hi", "from", " f", "in ",
    "de", "ion", "me", "v", ".", "ve", "all", "re ", "ri", "ro", "is ", "co", "f t", "are", "ea",
    ". ", "her", " m", "er ", " p", "es ", "by", "they", "di", "ra", "ic", "not", "s, ", "d t",
    "at ", "ce", "la", "h ", "ne", "as ", "tio", "on ", "n t", "io", "we", " a ", "om", ", a",
    "s o", "ur", "li", "ll", "ch", "had", "this", "e t", "g ", "e\r\n", " wh", "ere", " co", "e o",
    "a ", "us", " d", "ss", "\n\r\n", "\r\n\r", "=\"", " be", " e", "s a", "ma", "one", "t t",
    "or ", "but", "el", "so", "l ", "e s", "s,", "no", "ter", " wa", "iv", "ho", "e a", " r",
    "hat", "s t", "ns", "ch ", "wh", "tr", "ut", "/", "have", "ly ", "ta", " ha", " on", "tha",
    "-", " l", "ati", "en ", "pe", " re", "there", "ass", "si", " fo", "wa", "ec", "our", "who",
    "its", "z", "fo", "rs", ">", "ot", "un", "<", "im", "th ", "nc", "ate", "><", "ver", "ad",
    " we", "ly", "ee", " n", "id", " cl", "ac", "il", "</", "rt", " wi", "div", "e, ", " it",
    "whi", " ma", "ge", "x", "e c", "men", ".com",
];

/// The lowest and highest bytes that appear in the codebook. Each trie node
/// has one slot per byte in this range.
const TRIE_MIN_BYTE: u8 = 10;
const TRIE_MAX_BYTE: u8 = 122;
const TRIE_WIDTH: usize = (TRIE_MAX_BYTE - TRIE_MIN_BYTE + 1) as usize;

/// Marks a slot as a terminal node; the low byte holds the codebook index.
const TRIE_TERMINAL: u16 = 0x8000;

/// A trie node: the next row for each byte in the codebook range, and the
/// codebook index of the string ending at this node (255 if none).
type TrieNode = ([u16; TRIE_WIDTH], u8);

/// Build the codebook trie. Row 0 is the root. A slot is 0 if no codebook
/// entry continues with that byte, `TRIE_TERMINAL | index` if a codebook
/// entry ends there and none continue, and otherwise the row of the next
/// node.
fn build_codebook_trie() -> Vec<TrieNode> {
    let mut codebook_trie: Vec<TrieNode> = vec![([0; TRIE_WIDTH], 255)];
    for (i, entry) in CODEBOOK.iter().enumerate() {
        let mut row = 0;
        let s = entry.as_bytes();
        for &b in &s[..s.len() - 1] {
            let slot = (b - TRIE_MIN_BYTE) as usize;
            if codebook_trie[row].0[slot] == 0 {
                codebook_trie[row].0[slot] = codebook_trie.len() as u16;
                row = codebook_trie.len();
                codebook_trie.push(([0; TRIE_WIDTH], 255));
            } else if codebook_trie[row].0[slot] & TRIE_TERMINAL != 0 {
                // A shorter entry ended here; turn it into a node that keeps
                // its index
                let index = (codebook_trie[row].0[slot] & 0x00ff) as u8;
                codebook_trie[row].0[slot] = codebook_trie.len() as u16;
                row = codebook_trie.len();
                codebook_trie.push(([0; TRIE_WIDTH], index));
            } else {
                row = codebook_trie[row].0[slot] as usize;
            }
        }
        let slot = (s[s.len() - 1] - TRIE_MIN_BYTE) as usize;
        if codebook_trie[row].0[slot] == 0 {
            codebook_trie[row].0[slot] = TRIE_TERMINAL | i as u16;
        } else {
            row = codebook_trie[row].0[slot] as usize;
            codebook_trie[row].1 = i as u8;
        }
    }
    codebook_trie
}

/// Advance the trie by one byte.
///
/// # Arguments
/// * `row` - The current row in the trie.
/// * `byte` - The next byte of the input.
///
/// # Returns
/// A tuple of the next row and the code. If the row is zero then no
/// codebook entry continues past this byte. If the code is 255 then the
/// bytes read so far are not a complete codebook entry.
fn codebook_trie_next_char(trie: &[TrieNode], row: usize, byte: u8) -> (usize, u8) {
    if !(TRIE_MIN_BYTE..=TRIE_MAX_BYTE).contains(&byte) {
        return (0, 255);
    }
    let next = trie[row].0[(byte - TRIE_MIN_BYTE) as usize];
    if next == 0 {
        (0, 255)
    } else if next & TRIE_TERMINAL != 0 {
        (0, (next & 0x00ff) as u8)
    } else {
        (next as usize, trie[next as usize].1)
    }
}

lazy_static! {
    static ref CODEBOOK_TRIE: Vec<TrieNode> = build_codebook_trie();
}

/// The error type for decompress operation.
///
/// Often this error occurs due to invalid data.
#[derive(Debug, Clone, Copy)]
pub struct DecompressError;

impl fmt::Display for DecompressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid compressed data")
    }
}

impl Error for DecompressError {}

/// A specialized Result type for decompress operation.
pub type Result<T> = result::Result<T, DecompressError>;

/// Write a run of up to 256 bytes that are not in the codebook.
fn flush_verbatim(out: &mut Vec<u8>, verbatim: &[u8]) {
    if verbatim.len() > 1 {
        out.push(255);
        out.push((verbatim.len() - 1) as u8);
    } else {
        out.push(254);
    }
    out.extend_from_slice(verbatim);
}

/// Returns compressed data as a vector of bytes.
///
/// # Examples
///
/// ```
/// use smaz::compress;
///
/// let s = "string";
/// let compressed = compress(s.as_bytes());
/// assert_eq!(vec![77, 114, 84], compressed);
/// ```
pub fn compress(input: &[u8]) -> Vec<u8> {
    let trie: &[TrieNode] = &CODEBOOK_TRIE;
    let mut out: Vec<u8> = Vec::with_capacity(input.len() / 2);
    // Start of the current run of bytes that are not in the codebook
    let mut verbatim_start = 0;
    let mut input_index = 0;

    while input_index < input.len() {
        let max_len = (input.len() - input_index).min(7);

        let mut row = 0;
        let mut best_i = 0;
        let mut best_code = 0;
        for b in 0..max_len {
            let (new_row, code) = codebook_trie_next_char(trie, row, input[input_index + b]);
            if code != 255 {
                best_i = b + 1;
                best_code = code;
            }
            row = new_row;
            if row == 0 {
                break;
            }
        }

        if best_i > 0 {
            if verbatim_start < input_index {
                flush_verbatim(&mut out, &input[verbatim_start..input_index]);
            }
            out.push(best_code);
            input_index += best_i;
            verbatim_start = input_index;
        } else {
            input_index += 1;
            if input_index - verbatim_start == 256 {
                flush_verbatim(&mut out, &input[verbatim_start..input_index]);
                verbatim_start = input_index;
            }
        }
    }

    if verbatim_start < input_index {
        flush_verbatim(&mut out, &input[verbatim_start..input_index]);
    }
    out
}

/// Returns decompressed data as a vector of bytes.
///
/// # Errors
///
/// If the compressed data is invalid or encoded incorrectly, then an error
/// is returned [`DecompressError`](struct.DecompressError.html).
///
/// # Examples
///
/// ```
/// use std::str;
/// use smaz::decompress;
///
/// let v = vec![77, 114, 84];
/// let decompressed = decompress(&v).unwrap();
/// let origin = str::from_utf8(&decompressed).unwrap();
/// assert_eq!("string", origin);
/// ```
pub fn decompress(input: &[u8]) -> Result<Vec<u8>> {
    let mut out: Vec<u8> = Vec::with_capacity(input.len() * 3);
    let mut i: usize = 0;

    while i < input.len() {
        if input[i] == 254 {
            if i + 1 >= input.len() {
                return Err(DecompressError);
            }
            out.push(input[i + 1]);
            i += 2;
        } else if input[i] == 255 {
            if i + 2 + input[i + 1] as usize > input.len() {
                return Err(DecompressError);
            }
            for j in 0..=input[i + 1] {
                out.push(input[i + 2 + j as usize])
            }
            i += 3 + input[i + 1] as usize
        } else {
            for c in CODEBOOK[input[i] as usize].as_bytes().iter() {
                out.push(*c);
            }

            i += 1;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_STRINGS: [&str; 16] = [
        "",
        "This is a small string",
        "foobar",
        "the end",
        "not-a-g00d-Exampl333",
        "Smaz is a simple compression library",
        "Nothing is more difficult, and therefore more precious, than to be able to decide",
        "this is an example of what works very well with smaz",
        "1000 numbers 2000 will 10 20 30 compress very little",
        "and now a few italian sentences:",
        "Nel mezzo del cammin di nostra vita, mi ritrovai in una selva oscura",
        "Mi illumino di immenso",
        "L'autore di questa libreria vive in Sicilia",
        "try it against urls",
        "http://google.com",
        "http://programming.reddit.com",
    ];

    #[test]
    fn test_compress() {
        for s in TEST_STRINGS.iter() {
            let compressed = compress(s.as_bytes());
            let decompressed = decompress(&compressed);

            if let Ok(v) = decompressed {
                assert_eq!(v, s.to_string().into_bytes());
            } else {
                panic!("Could not decompress string {}.", s);
            }

            if !s.is_empty() {
                let level = 100i8 - ((100 * compressed.len()) / s.len()) as i8;
                let word = if level > 0 { "compressed" } else { "enlarged" };
                println!("\"{}\" {} by {}%", s, word, level.abs());
            }
        }
    }

    fn codebook_trie_get(s: &str) -> Option<u8> {
        let mut row = 0;
        let s = s.as_bytes();
        for &b in &s[..s.len() - 1] {
            let (new_row, _) = codebook_trie_next_char(&CODEBOOK_TRIE, row, b);
            if new_row == 0 {
                return None;
            }
            row = new_row;
        }
        let (_, new_code) = codebook_trie_next_char(&CODEBOOK_TRIE, row, s[s.len() - 1]);
        if new_code == 255 {
            None
        } else {
            Some(new_code)
        }
    }

    #[test]
    fn test_codebook_trie_get() {
        for (i, entry) in CODEBOOK.iter().enumerate() {
            assert_eq!(Some(i as u8), codebook_trie_get(entry));
        }
        assert_eq!(None, codebook_trie_get("foobar"));
    }
}
