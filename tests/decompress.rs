use smaz::{compress, decompress};

/// Small deterministic pseudo-random generator, so the tests need no extra
/// dependencies and failures are reproducible.
struct Lcg(u64);

impl Lcg {
    fn next_u8(&mut self) -> u8 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u8
    }

    fn bytes(&mut self, max_len: usize) -> Vec<u8> {
        let len = (self.next_u8() as usize * 256 + self.next_u8() as usize) % (max_len + 1);
        (0..len).map(|_| self.next_u8()).collect()
    }
}

#[test]
fn truncated_input_is_an_error() {
    // 254 is followed by one verbatim byte
    assert!(decompress(&[254]).is_err());
    assert!(decompress(&[77, 254]).is_err());
    // 255 is followed by a length byte n and then n + 1 verbatim bytes
    assert!(decompress(&[255]).is_err());
    assert!(decompress(&[77, 255]).is_err());
    assert!(decompress(&[255, 0]).is_err());
    assert!(decompress(&[255, 3, b'a']).is_err());
    assert!(decompress(&[255, 3, b'a', b'b', b'c']).is_err());
}

#[test]
fn complete_verbatim_input_is_decoded() {
    assert_eq!(decompress(&[254, b'a']).unwrap(), b"a");
    assert_eq!(decompress(&[255, 0, b'a']).unwrap(), b"a");
    assert_eq!(
        decompress(&[255, 3, b'a', b'b', b'c', b'd']).unwrap(),
        b"abcd"
    );
}

#[test]
fn decompress_never_panics_on_short_input() {
    // Every input of up to two bytes
    decompress(&[]).unwrap();
    for a in 0..=255u8 {
        let _ = decompress(&[a]);
        for b in 0..=255u8 {
            let _ = decompress(&[a, b]);
        }
    }
}

#[test]
fn decompress_never_panics_on_random_input() {
    let mut rng = Lcg(42);
    for _ in 0..100_000 {
        let _ = decompress(&rng.bytes(300));
    }
}

#[test]
fn round_trip_random_input() {
    let mut rng = Lcg(7);
    for _ in 0..20_000 {
        let input = rng.bytes(600);
        assert_eq!(decompress(&compress(&input)).unwrap(), input);
    }
}

#[test]
fn round_trip_long_verbatim_runs() {
    // Bytes >= 128 never match the codebook, so these are encoded verbatim,
    // in chunks of at most 256 bytes
    for len in [1, 2, 255, 256, 257, 511, 512, 513, 2000] {
        let input: Vec<u8> = (0..len).map(|i| 128 + (i % 128) as u8).collect();
        assert_eq!(decompress(&compress(&input)).unwrap(), input);
    }
}
