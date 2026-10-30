//! Checks that the trie-based compressor produces exactly the same output as
//! the original hash-map implementation (smaz 0.1.0 on crates.io), so data
//! compressed by either can be read by the other.

use smaz::{CODEBOOK, compress};
use smaz_reference::compress as reference_compress;

struct Lcg(u64);

impl Lcg {
    fn next_u8(&mut self) -> u8 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u8
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next_u8() as usize * 256 + self.next_u8() as usize) % n
    }
}

fn assert_same(input: &[u8]) {
    assert_eq!(
        compress(input),
        reference_compress(input),
        "different output for {:?}",
        String::from_utf8_lossy(input)
    );
}

#[test]
fn same_output_for_short_input() {
    assert_same(&[]);
    for a in 0..=255u8 {
        assert_same(&[a]);
        for b in 0..=255u8 {
            assert_same(&[a, b]);
        }
    }
}

#[test]
fn same_output_for_codebook_word_sequences() {
    // Concatenated codebook entries overlap in many ways, which exercises
    // the longest-match search
    let mut rng = Lcg(1);
    for _ in 0..20_000 {
        let n = 1 + rng.below(12);
        let s: String = (0..n)
            .map(|_| CODEBOOK[rng.below(CODEBOOK.len())])
            .collect();
        assert_same(s.as_bytes());
    }
}

#[test]
fn same_output_for_random_text() {
    let alphabet = b"etaoinshrdlucmfwypvbgkjqxz ETAOINSHRDLU.,;:!?'\"-<>/=0123456789\r\n\t";
    let mut rng = Lcg(2);
    for _ in 0..20_000 {
        let len = rng.below(200);
        let s: Vec<u8> = (0..len)
            .map(|_| alphabet[rng.below(alphabet.len())])
            .collect();
        assert_same(&s);
    }
}

#[test]
fn same_output_for_random_bytes() {
    let mut rng = Lcg(3);
    for _ in 0..20_000 {
        let len = rng.below(600);
        let s: Vec<u8> = (0..len).map(|_| rng.next_u8()).collect();
        assert_same(&s);
    }
}
