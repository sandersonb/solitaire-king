//! Human-friendly encoding of the 64-bit deal seed.
//!
//! A raw `u64` seed is reproducible but unmemorable and awkward to share by
//! voice. We encode it as a pronounceable [proquint] string: a `u64` becomes
//! four five-letter "quints" joined by dashes, e.g. `lusab-babad-gutih-tugad`.
//! Each quint packs 16 bits as consonant-vowel-consonant-vowel-consonant, so the
//! whole `u64` round-trips exactly.
//!
//! [`decode`] also accepts a plain decimal `u64`, so seeds recorded before this
//! encoding existed keep working. The mapping to a deal is unchanged — this is a
//! presentation/parsing layer over the same `u64`.
//!
//! [proquint]: https://arxiv.org/html/0901.4016

/// 16 consonants encode 4 bits each.
const CONSONANTS: [u8; 16] = *b"bdfghjklmnprstvz";
/// 4 vowels encode 2 bits each.
const VOWELS: [u8; 4] = *b"aiou";

/// Encode a 16-bit value as one five-letter quint (C V C V C).
fn encode_quint(n: u16, out: &mut String) {
    // Emit most-significant field first so the string reads high→low.
    out.push(CONSONANTS[(n >> 12) as usize & 0xF] as char);
    out.push(VOWELS[(n >> 10) as usize & 0x3] as char);
    out.push(CONSONANTS[(n >> 6) as usize & 0xF] as char);
    out.push(VOWELS[(n >> 4) as usize & 0x3] as char);
    out.push(CONSONANTS[n as usize & 0xF] as char);
}

/// Encode a `u64` seed as a pronounceable proquint string (four dashed quints).
pub fn encode(seed: u64) -> String {
    let mut s = String::with_capacity(23);
    for i in 0..4 {
        if i > 0 {
            s.push('-');
        }
        let quint = (seed >> (48 - i * 16)) as u16;
        encode_quint(quint, &mut s);
    }
    s
}

fn consonant_bits(c: u8) -> Option<u16> {
    CONSONANTS.iter().position(|&x| x == c).map(|i| i as u16)
}

fn vowel_bits(c: u8) -> Option<u16> {
    VOWELS.iter().position(|&x| x == c).map(|i| i as u16)
}

/// Decode 20 compacted (dash/space-free, lowercased) proquint chars to a `u64`.
fn decode_proquint(compact: &[u8]) -> Option<u64> {
    if compact.len() != 20 {
        return None;
    }
    let mut seed: u64 = 0;
    for quint in compact.chunks(5) {
        let n = (consonant_bits(quint[0])? << 12)
            | (vowel_bits(quint[1])? << 10)
            | (consonant_bits(quint[2])? << 6)
            | (vowel_bits(quint[3])? << 4)
            | consonant_bits(quint[4])?;
        seed = (seed << 16) | n as u64;
    }
    Some(seed)
}

/// Decode a seed string to its `u64`.
///
/// Accepts a proquint string (case-insensitive; dashes and whitespace ignored)
/// or a plain decimal `u64`. Returns `None` for anything else, so an invalid
/// seed is rejected rather than silently producing the wrong deal.
pub fn decode(input: &str) -> Option<u64> {
    let compact: Vec<u8> = input
        .bytes()
        .filter(|b| !b.is_ascii_whitespace() && *b != b'-')
        .map(|b| b.to_ascii_lowercase())
        .collect();
    if let Some(v) = decode_proquint(&compact) {
        return Some(v);
    }
    input.trim().parse::<u64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_many_seeds() {
        let mut samples = vec![
            0u64,
            1,
            42,
            2024,
            u64::MAX,
            u64::MAX - 1,
            0x0123_4567_89AB_CDEF,
        ];
        // A spread of pseudo-random values via a simple LCG.
        let mut x = 0x9E37_79B9_7F4A_7C15u64;
        for _ in 0..1000 {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            samples.push(x);
        }
        for s in samples {
            let enc = encode(s);
            assert_eq!(decode(&enc), Some(s), "round trip failed for {s} ({enc})");
        }
    }

    #[test]
    fn encoding_is_stable_and_shaped() {
        let e = encode(0);
        assert_eq!(e, "babab-babab-babab-babab");
        // Four five-letter quints joined by three dashes.
        assert_eq!(e.len(), 23);
        assert_eq!(e.matches('-').count(), 3);
    }

    #[test]
    fn distinct_seeds_encode_distinctly() {
        assert_ne!(encode(1), encode(2));
    }

    #[test]
    fn raw_u64_fallback() {
        assert_eq!(decode("42"), Some(42));
        assert_eq!(decode("  2024 "), Some(2024));
        assert_eq!(decode("18446744073709551615"), Some(u64::MAX));
    }

    #[test]
    fn case_insensitive_and_separator_tolerant() {
        let e = encode(0xDEAD_BEEF_1234_5678);
        assert_eq!(decode(&e.to_uppercase()), Some(0xDEAD_BEEF_1234_5678));
        let spaced = e.replace('-', " ");
        assert_eq!(decode(&spaced), Some(0xDEAD_BEEF_1234_5678));
        let undashed = e.replace('-', "");
        assert_eq!(decode(&undashed), Some(0xDEAD_BEEF_1234_5678));
    }

    #[test]
    fn rejects_invalid_input() {
        assert_eq!(decode(""), None);
        assert_eq!(decode("not-a-seed"), None); // valid length shape but bad letters
        assert_eq!(decode("xyzzy"), None);
        assert_eq!(decode("18446744073709551616"), None); // u64::MAX + 1 overflows
        assert_eq!(decode("lusab-babad-gutih"), None); // too short for proquint, not a u64
    }
}
