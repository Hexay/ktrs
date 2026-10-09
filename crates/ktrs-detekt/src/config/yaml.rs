//! The YAML subset detekt configs are written in, loaded like snakeyaml-engine's `Load` with the core schema:
//! block mappings and sequences by indentation, one-line flow sequences, plain and quoted scalars, comments.
//!
//! TODO (research/33 decision 8): replace with an event-level YAML parser plus a port of snakeyaml-engine's
//! resolver. Not handled here, and reported as an error: anchors and aliases, tags, block scalars (`|`, `>`),
//! flow mappings other than `{}`, flow collections over several lines, multi-line scalars, several documents.

use crate::api::Value;

struct Line<'a> {
    number: usize,
    indent: usize,
    text: &'a str,
}

/// `Load.loadFromReader`: the document's root; `Ok(None)` for an empty document.
pub fn load(text: &str) -> Result<Option<Value>, String> {
    let mut lines = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let content = strip_comment(raw).trim_end();
        if content.trim_start().is_empty() {
            continue;
        }
        if content == "---" || content == "..." {
            return Err(format!("line {}: several documents are not supported", index + 1));
        }
        let indent = content.len() - content.trim_start_matches(' ').len();
        lines.push(Line { number: index + 1, indent, text: &content[indent..] });
    }
    if lines.is_empty() {
        return Ok(None);
    }
    let mut position = 0;
    let root = parse_block(&lines, &mut position, lines[0].indent)?;
    match lines.get(position) {
        Some(line) => Err(format!("line {}: unexpected indentation", line.number)),
        None => Ok(Some(root)),
    }
}

/// The text before a `#` comment: a `#` at the line start or after a space, outside quotes.
fn strip_comment(line: &str) -> &str {
    let mut quote = None;
    let mut previous = ' ';
    for (index, c) in line.char_indices() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if c == '\'' || c == '"' => quote = Some(c),
            None if c == '#' && matches!(previous, ' ' | '\t') => return &line[..index],
            None => {}
        }
        previous = c;
    }
    line
}

fn parse_block(lines: &[Line], position: &mut usize, indent: usize) -> Result<Value, String> {
    if is_sequence_item(lines[*position].text) { parse_sequence(lines, position, indent) } else { parse_mapping(lines, position, indent) }
}

fn is_sequence_item(text: &str) -> bool {
    text == "-" || text.starts_with("- ")
}

fn parse_sequence(lines: &[Line], position: &mut usize, indent: usize) -> Result<Value, String> {
    let mut items = Vec::new();
    while let Some(line) = lines.get(*position).filter(|l| l.indent == indent && is_sequence_item(l.text)) {
        let rest = line.text[1..].trim_start();
        if rest.is_empty() {
            *position += 1;
            items.push(parse_nested(lines, position, indent, line.number)?);
        } else if split_key(rest).is_some() {
            // `- key: value`: a mapping whose first entry shares the dash's line.
            let item_indent = indent + (line.text.len() - rest.len());
            let first = Line { number: line.number, indent: item_indent, text: rest };
            let mut entries = Vec::new();
            *position += 1;
            parse_entry(&first, lines, position, item_indent, &mut entries)?;
            parse_entries(lines, position, item_indent, &mut entries)?;
            items.push(Value::Map(entries));
        } else {
            *position += 1;
            items.push(parse_inline(rest, line.number)?);
        }
    }
    Ok(Value::List(items))
}

fn parse_mapping(lines: &[Line], position: &mut usize, indent: usize) -> Result<Value, String> {
    let mut entries = Vec::new();
    parse_entries(lines, position, indent, &mut entries)?;
    Ok(Value::Map(entries))
}

fn parse_entries(lines: &[Line], position: &mut usize, indent: usize, entries: &mut Vec<(String, Value)>) -> Result<(), String> {
    while let Some(line) = lines.get(*position).filter(|l| l.indent == indent && !is_sequence_item(l.text)) {
        *position += 1;
        parse_entry(line, lines, position, indent, entries)?;
    }
    Ok(())
}

/// One `key: value` line (`position` is past it): the value is inline, a nested block, or absent (null: dropped,
/// as `Map<String, Any>` reads it).
fn parse_entry(line: &Line, lines: &[Line], position: &mut usize, indent: usize, entries: &mut Vec<(String, Value)>) -> Result<(), String> {
    let Some((key, value)) = split_key(line.text) else {
        return Err(format!("line {}: expected `key: value`", line.number));
    };
    let key = match parse_inline(key, line.number)? {
        Value::String(key) => key,
        other => other.render(),
    };
    if entries.iter().any(|(k, _)| *k == key) {
        return Err(format!("line {}: found duplicate key {key}", line.number));
    }
    if !value.is_empty() {
        if !is_null(value) {
            entries.push((key, parse_inline(value, line.number)?));
        }
        return Ok(());
    }
    let nested_is_sequence_at_key_indent = lines.get(*position).is_some_and(|l| l.indent == indent && is_sequence_item(l.text));
    if nested_is_sequence_at_key_indent {
        entries.push((key, parse_sequence(lines, position, indent)?));
    } else if lines.get(*position).is_some_and(|l| l.indent > indent) {
        entries.push((key, parse_nested(lines, position, indent, line.number)?));
    }
    Ok(())
}

/// The block nested under the line before `position`, which must be indented deeper than `indent`.
fn parse_nested(lines: &[Line], position: &mut usize, indent: usize, number: usize) -> Result<Value, String> {
    match lines.get(*position) {
        Some(next) if next.indent > indent => parse_block(lines, position, next.indent),
        _ => Err(format!("line {number}: expected a nested block")),
    }
}

/// Splits `key: value` at the first `:` followed by a space or the line end, outside quotes.
fn split_key(text: &str) -> Option<(&str, &str)> {
    if text.starts_with(['[', '{']) {
        return None;
    }
    let mut quote = None;
    for (index, c) in text.char_indices() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if c == '\'' || c == '"' => quote = Some(c),
            None if c == ':' && (text[index + 1..].is_empty() || text[index + 1..].starts_with(' ')) => {
                return Some((text[..index].trim_end(), text[index + 1..].trim_start()));
            }
            None => {}
        }
    }
    None
}

fn parse_inline(text: &str, number: usize) -> Result<Value, String> {
    if let Some(inner) = text.strip_prefix('[') {
        let Some(inner) = inner.strip_suffix(']') else {
            return Err(format!("line {number}: flow sequences over several lines are not supported"));
        };
        let items = split_flow_items(inner);
        return items.into_iter().filter(|item| !is_null(item)).map(|item| parse_inline(item, number)).collect::<Result<_, _>>().map(Value::List);
    }
    if text == "{}" {
        return Ok(Value::Map(Vec::new()));
    }
    if text.starts_with(['{', '&', '*', '!', '|', '>']) {
        return Err(format!("line {number}: unsupported YAML syntax `{text}`"));
    }
    if let Some(inner) = text.strip_prefix('\'') {
        let Some(inner) = inner.strip_suffix('\'') else { return Err(format!("line {number}: unterminated quoted scalar")) };
        return Ok(Value::String(inner.replace("''", "'")));
    }
    if let Some(inner) = text.strip_prefix('"') {
        let Some(inner) = inner.strip_suffix('"') else { return Err(format!("line {number}: unterminated quoted scalar")) };
        return unescape_double_quoted(inner).map(Value::String).ok_or_else(|| format!("line {number}: unsupported escape"));
    }
    Ok(resolve_plain_scalar(text))
}

fn split_flow_items(inner: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let (mut quote, mut start) = (None, 0);
    for (index, c) in inner.char_indices() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if c == '\'' || c == '"' => quote = Some(c),
            None if c == ',' => {
                items.push(inner[start..index].trim());
                start = index + 1;
            }
            None => {}
        }
    }
    let last = inner[start..].trim();
    if !last.is_empty() || !items.is_empty() {
        items.push(last);
    }
    items
}

fn unescape_double_quoted(inner: &str) -> Option<String> {
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        out.push(match chars.next()? {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            '0' => '\0',
            '"' => '"',
            '/' => '/',
            '\\' => '\\',
            ' ' => ' ',
            _ => return None,
        });
    }
    Some(out)
}

fn is_null(text: &str) -> bool {
    matches!(text, "" | "~" | "null" | "Null" | "NULL")
}

/// The core schema's implicit types: bool, int (decimal, `0o`, `0x`), float; anything else is a string.
fn resolve_plain_scalar(text: &str) -> Value {
    match text {
        "true" | "True" | "TRUE" => return Value::Boolean(true),
        "false" | "False" | "FALSE" => return Value::Boolean(false),
        _ => {}
    }
    let digits = text.strip_prefix(['-', '+']).unwrap_or(text);
    let integer = if let Some(octal) = text.strip_prefix("0o") {
        i64::from_str_radix(octal, 8).ok()
    } else if let Some(hex) = text.strip_prefix("0x") {
        i64::from_str_radix(hex, 16).ok()
    } else if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
        text.parse::<i64>().ok()
    } else {
        None
    };
    if let Some(integer) = integer {
        return i32::try_from(integer).map_or(Value::Long(integer), Value::Int);
    }
    let float_like = digits.bytes().any(|b| b.is_ascii_digit())
        && digits.bytes().all(|b| b.is_ascii_digit() || matches!(b, b'.' | b'e' | b'E' | b'-' | b'+'))
        && digits.contains(['.', 'e', 'E']);
    match text.parse::<f64>() {
        Ok(float) if float_like => Value::Double(float),
        _ => Value::String(text.to_owned()),
    }
}
