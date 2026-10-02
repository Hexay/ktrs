//! `PrintWriter.printf(text)` without arguments, as ktlint 1.8 writes `--stdin --format` output (#3281):
//! `java.util.Formatter` parses the whole text first (conversion errors win), then formats it, so `%%` and
//! `%n` are rewritten and any conversion that needs an argument fails. `Err` is the exception as
//! `<class>: <message>`; nothing is written then (the `PrintWriter` is never flushed).

use crate::ktlint::console::LINE_SEPARATOR;

const FLAGS: &str = "-#+ 0,(<";
/// `Formatter.Conversion.isValid`; `t`/`T` (date/time) take a second character.
const CONVERSIONS: &str = "bBhHsScCdoxXeEfgGaA%n";
const DATE_TIME_CONVERSIONS: &str = "HIklMSLNpzZsQBbhAaCYyjmdeRTrDFc";

enum Part<'a> {
    Fixed(&'a str),
    Specifier(Specifier),
}

struct Specifier {
    text: String,
    flags: String,
    width: Option<u32>,
    precision: Option<u32>,
    date_time: bool,
    conversion: char,
}

pub fn java_printf(text: &str) -> Result<String, String> {
    let mut out = String::new();
    for part in parse(text)? {
        match part {
            Part::Fixed(s) => out.push_str(s),
            Part::Specifier(s) if s.conversion == 'n' => out.push_str(LINE_SEPARATOR),
            Part::Specifier(s) if s.conversion == '%' => {
                let width = s.width.unwrap_or(0) as usize;
                if s.flags.contains('-') { out.push_str(&format!("{:<width$}", "%")) } else { out.push_str(&format!("{:>width$}", "%")) }
            }
            Part::Specifier(s) => return Err(format!("java.util.MissingFormatArgumentException: Format specifier '{}'", s.text)),
        }
    }
    Ok(out)
}

fn parse(text: &str) -> Result<Vec<Part<'_>>, String> {
    let unknown = |conversion: &str| format!("java.util.UnknownFormatConversionException: Conversion = '{conversion}'");
    let mut parts = Vec::new();
    let mut rest = text;
    while let Some(n) = rest.find('%') {
        if n > 0 {
            parts.push(Part::Fixed(&rest[..n]));
        }
        let after = &rest[n + 1..];
        let first = after.chars().next().ok_or_else(|| unknown("%"))?;
        let (specifier, length) = match parse_specifier(after) {
            Some(parsed) => parsed,
            None => return Err(unknown(&first.to_string())),
        };
        check(&specifier)?;
        parts.push(Part::Specifier(specifier));
        rest = &after[length..];
    }
    if !rest.is_empty() {
        parts.push(Part::Fixed(rest));
    }
    Ok(parts)
}

/// `%(\d+\$)?([-#+ 0,(<]*)?(\d+)?(\.\d+)?([tT])?([a-zA-Z%])` after the `%`; the specifier and its length.
fn parse_specifier(s: &str) -> Option<(Specifier, usize)> {
    let bytes = s.as_bytes();
    let digits = |from: usize| bytes[from..].iter().take_while(|b| b.is_ascii_digit()).count();
    let mut i = 0;
    let index_digits = digits(0);
    if index_digits > 0 && bytes.get(index_digits) == Some(&b'$') {
        i = index_digits + 1;
    }
    let flags_start = i;
    while bytes.get(i).is_some_and(|b| FLAGS.as_bytes().contains(b)) {
        i += 1;
    }
    let flags = s[flags_start..i].to_owned();
    let width_digits = digits(i);
    let width = (width_digits > 0).then(|| s[i..i + width_digits].parse().unwrap_or(u32::MAX));
    i += width_digits;
    let mut precision = None;
    if bytes.get(i) == Some(&b'.') {
        let precision_digits = digits(i + 1);
        if precision_digits == 0 {
            return None;
        }
        precision = Some(s[i + 1..i + 1 + precision_digits].parse().unwrap_or(u32::MAX));
        i += 1 + precision_digits;
    }
    let date_time = matches!(bytes.get(i), Some(b't' | b'T'));
    if date_time {
        i += 1;
    }
    let conversion = s[i..].chars().next().filter(|c| c.is_ascii_alphabetic() || *c == '%')?;
    i += 1;
    let specifier = Specifier { text: format!("%{}", &s[..i]), flags, width, precision, date_time, conversion };
    Some((specifier, i))
}

/// The checks `FormatSpecifier`'s constructor makes.
fn check(s: &Specifier) -> Result<(), String> {
    let unknown = |conversion: String| Err(format!("java.util.UnknownFormatConversionException: Conversion = '{conversion}'"));
    if s.date_time {
        return if DATE_TIME_CONVERSIONS.contains(s.conversion) { Ok(()) } else { unknown(format!("t{}", s.conversion)) };
    }
    if !CONVERSIONS.contains(s.conversion) {
        return unknown(s.conversion.to_string());
    }
    match s.conversion {
        'n' if s.precision.is_some() => Err(format!("java.util.IllegalFormatPrecisionException: {}", s.precision.unwrap())),
        'n' if s.width.is_some() => Err(format!("java.util.IllegalFormatWidthException: {}", s.width.unwrap())),
        'n' if !s.flags.is_empty() => Err(format!("java.util.IllegalFormatFlagsException: Flags = '{}'", s.flags)),
        '%' if s.precision.is_some() => Err(format!("java.util.IllegalFormatPrecisionException: {}", s.precision.unwrap())),
        '%' if s.flags.chars().any(|c| c != '-') => Err(format!("java.util.IllegalFormatFlagsException: Flags = '{}'", s.flags)),
        '%' if s.flags.contains('-') && s.width.is_none() => Err(format!("java.util.MissingFormatWidthException: {}", s.text)),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_text_conversions_and_rejects_the_rest() {
        assert_eq!(java_printf("val a = \"x\"\n"), Ok("val a = \"x\"\n".to_owned()));
        assert_eq!(java_printf("100%% %5%|%-3%|"), Ok("100%     %|%  |".to_owned()));
        assert_eq!(java_printf("a%nb"), Ok(format!("a{LINE_SEPARATOR}b")));
        assert_eq!(java_printf("\"%d %s\""), Err("java.util.MissingFormatArgumentException: Format specifier '%d'".to_owned()));
        assert_eq!(java_printf("%-5.2f"), Err("java.util.MissingFormatArgumentException: Format specifier '%-5.2f'".to_owned()));
        assert_eq!(java_printf("%1$s"), Err("java.util.MissingFormatArgumentException: Format specifier '%1$s'".to_owned()));
        // The whole text is parsed before anything is formatted.
        assert_eq!(java_printf("%d \"100%\""), Err("java.util.UnknownFormatConversionException: Conversion = '\"'".to_owned()));
        assert_eq!(java_printf("50%"), Err("java.util.UnknownFormatConversionException: Conversion = '%'".to_owned()));
        assert_eq!(java_printf("%q"), Err("java.util.UnknownFormatConversionException: Conversion = 'q'".to_owned()));
        assert_eq!(java_printf("%tq"), Err("java.util.UnknownFormatConversionException: Conversion = 'tq'".to_owned()));
        assert_eq!(java_printf("%tY"), Err("java.util.MissingFormatArgumentException: Format specifier '%tY'".to_owned()));
        assert_eq!(java_printf("%-%"), Err("java.util.MissingFormatWidthException: %-%".to_owned()));
    }
}
