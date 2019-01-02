#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::result;
use std::str;

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

lazy_static! {
    static ref CODEBOOK_MAP: HashMap<Vec<u8>, u8> = {
        let mut map: HashMap<Vec<u8>, u8> = HashMap::new();
        for (i, code) in CODEBOOK.iter().enumerate() {
            map.insert(code.to_string().into_bytes(), i as u8);
        }
        map
    };
}

#[derive(Debug, Clone)]
pub struct DecompressError;

impl fmt::Display for DecompressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid compressed data")
    }
}

impl Error for DecompressError {
    fn description(&self) -> &str {
        "invalid compressed data"
    }
}

pub type Result<T> = result::Result<T, DecompressError>;

fn flush_verbatim(verbatim: &[u8]) -> Vec<u8> {
    let mut chunk: Vec<u8> = Vec::new();
    if verbatim.len() > 1 {
        chunk.push(255);
        chunk.push((verbatim.len() - 1) as u8);
    } else {
        chunk.push(254);
    }
    for c in verbatim {
        chunk.push(*c)
    }
    chunk
}

pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len()/2);
    let mut verbatim: Vec<u8> = Vec::new();
    let mut input_index = 0;

    while input_index < input.len() {
        let mut encoded = false;
        let mut max_len = 7;
        if (input.len() - input_index) < 7 {
            max_len = input.len() - input_index
        }

        for i in (0..max_len + 1).rev() {
            let code = CODEBOOK_MAP.get(&input[input_index..input_index + i]);
            if let Some(v) = code {
                if !verbatim.is_empty() {
                    out.append(&mut flush_verbatim(&verbatim));
                    verbatim.clear();
                }
                out.push(v.clone());
                input_index += i;
                encoded = true;
                break;
            }
        }

        if !encoded {
            verbatim.push(input[input_index]);
            input_index += 1;

            if verbatim.len() == 256 {
                out.append(&mut flush_verbatim(&verbatim));
                verbatim.clear();
            }
        }
    }

    if !verbatim.is_empty() {
        out.append(&mut flush_verbatim(&verbatim));
    }
    out
}

pub fn decompress(input: &[u8]) -> Result<Vec<u8>> {
    let mut out: Vec<u8> = Vec::with_capacity(input.len()*3);
    let mut i: usize = 0;

    while i < input.len() {
        if input[i] == 254 {
            if i + 1 > input.len() {
                return Err(DecompressError);
            }
            out.push(input[i + 1]);
            i += 2;
        } else if input[i] == 255 {
            if i + input[i + 1] as usize + 2 >= input.len() {
                return Err(DecompressError);
            }
            for j in 0..input[i + 1] + 1 {
                out.push(input[i + 2 + j as usize])
            }
            i += 3 + input[i + 1] as usize
        } else {
            for c in CODEBOOK[input[i] as usize].as_bytes().iter() {
                out.push(c.clone());
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
            let compressed = compress(&s.as_bytes());
            let decompressed = decompress(&compressed);

            if let Ok(v) = decompressed {
                assert_eq!(v, s.to_string().into_bytes());
            } else {
                panic!("Could not decompress string {}.", s);
            }

            if s.len() > 0 {
                let level = 100i8 - ((100 * compressed.len()) / s.as_bytes().len()) as i8;
                let word = if level > 0 {"compressed"} else {"enlarged"};
                println!("{} {} by {}%", s, word, level.abs());
            }
        }
    }
}
