//! Kotlin's `String.toIntOrNull()` / `toInt()` (Java's `Integer.parseInt`), which the range flags' values go through.

/// An optional sign, then digits in the sense of `Character.digit(char, 10)`.
pub(crate) fn to_int_or_null(value: &str) -> Option<i32> {
    let (sign, digits) = match value.strip_prefix(['+', '-']) {
        Some(digits) => (&value[..1], digits),
        None => ("", value),
    };
    if digits.is_empty() {
        return None;
    }
    let ascii: Option<String> = digits.chars().map(|c| java_digit(c).map(|d| (b'0' + d) as char)).collect();
    format!("{sign}{}", ascii?).parse().ok()
}

/// The zero of every decimal digit run in the BMP (Java 21's `Character.digit`; a `char` there is one UTF-16 unit).
const DIGIT_ZEROS: [u32; 37] = [
    0x0030, 0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6, 0x0D66, 0x0DE6,
    0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80, 0x1A90, 0x1B50, 0x1BB0, 0x1C40,
    0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0, 0xFF10,
];

fn java_digit(c: char) -> Option<u8> {
    let zero = DIGIT_ZEROS[DIGIT_ZEROS.partition_point(|&zero| zero <= c as u32).checked_sub(1)?];
    u8::try_from(c as u32 - zero).ok().filter(|&d| d < 10)
}

#[cfg(test)]
mod tests {
    use super::to_int_or_null;

    #[test]
    fn parses_like_integer_parse_int() {
        for (value, expected) in [
            ("5", Some(5)),
            ("+5", Some(5)),
            ("-5", Some(-5)),
            ("\u{665}", Some(5)),
            ("\u{ff11}\u{ff10}", Some(10)),
            ("-2147483648", Some(i32::MIN)),
            ("2147483648", None),
            ("+", None),
            ("", None),
            (" 5", None),
            ("1_0", None),
            ("\u{1d7ce}", None),
        ] {
            assert_eq!(to_int_or_null(value), expected, "{value:?}");
        }
    }
}
