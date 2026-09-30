//! The slice of Clikt's parser that ktlint's commands use: long/short options, attached values, short
//! flag groups, `--`, `@argfile`s, subcommands by name, and Clikt's error messages and typo suggestions.

use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arity {
    Flag,
    Value,
    /// `optionalValue(acceptsUnattachedValue = false)`: only `--name=value` takes a value.
    OptionalAttached,
}

pub struct OptionSpec {
    pub names: &'static [&'static str],
    pub arity: Arity,
    pub hidden: bool,
}

/// An option occurrence: its canonical (first) name and value.
#[derive(Clone, Debug)]
pub struct Invocation {
    pub name: &'static str,
    pub value: Option<String>,
}

pub struct ParsedTokens {
    pub invocations: Vec<Invocation>,
    pub arguments: Vec<String>,
    /// The subcommand name and the tokens after it.
    pub subcommand: Option<(&'static str, Vec<String>)>,
}

/// Parses `tokens` against `options`; `Err` is a Clikt usage error message (without `Error: `).
pub fn parse_tokens(
    tokens: &[String],
    options: &'static [OptionSpec],
    subcommands: &[&'static str],
) -> Result<ParsedTokens, String> {
    let mut parsed = ParsedTokens { invocations: Vec::new(), arguments: Vec::new(), subcommand: None };
    let find = |name: &str| options.iter().find(|o| o.names.contains(&name));
    let mut i = 0;
    let mut only_arguments = false;
    while i < tokens.len() {
        let token = &tokens[i];
        i += 1;
        if only_arguments || token == "-" || !token.starts_with('-') {
            if !only_arguments && let Some(sub) = subcommands.iter().find(|s| **s == token) {
                parsed.subcommand = Some((sub, tokens[i..].to_vec()));
                break;
            }
            parsed.arguments.push(token.clone());
        } else if token == "--" {
            only_arguments = true;
        } else if token.starts_with("--") {
            let (name, attached) = match token.split_once('=') {
                Some((name, value)) => (name, Some(value.to_owned())),
                None => (token.as_str(), None),
            };
            let spec = find(name).ok_or_else(|| no_such_option(name, options))?;
            let value = match (spec.arity, attached) {
                (Arity::Flag, Some(_)) => return Err(format!("option {name} does not take a value")),
                (Arity::Flag, None) => None,
                (Arity::OptionalAttached, attached) => Some(attached.unwrap_or_default()),
                (Arity::Value, Some(value)) => Some(value),
                (Arity::Value, None) => match tokens.get(i) {
                    Some(value) => {
                        i += 1;
                        Some(value.clone())
                    }
                    None => return Err(format!("option {name} requires a value")),
                },
            };
            parsed.invocations.push(Invocation { name: spec.names[0], value });
        } else {
            let chars: Vec<char> = token.chars().skip(1).collect();
            for (k, c) in chars.iter().enumerate() {
                let name = format!("-{c}");
                let spec = find(&name).ok_or_else(|| no_such_option(&name, options))?;
                if spec.arity == Arity::Flag {
                    parsed.invocations.push(Invocation { name: spec.names[0], value: None });
                    continue;
                }
                let rest: String = chars[k + 1..].iter().collect();
                let value = if !rest.is_empty() {
                    rest
                } else if let Some(value) = tokens.get(i) {
                    i += 1;
                    value.clone()
                } else {
                    return Err(format!("option {name} requires a value"));
                };
                parsed.invocations.push(Invocation { name: spec.names[0], value: Some(value) });
                break;
            }
        }
    }
    Ok(parsed)
}

fn no_such_option(name: &str, options: &[OptionSpec]) -> String {
    let mut scored: Vec<(&str, f64)> = options
        .iter()
        .filter(|o| !o.hidden)
        .flat_map(|o| o.names.iter())
        .map(|candidate| (*candidate, jaro_winkler_similarity(name, candidate)))
        .filter(|(_, similarity)| *similarity > 0.8)
        .collect();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    match scored.as_slice() {
        [] => format!("no such option {name}"),
        [(only, _)] => format!("no such option {name}. Did you mean {only}?"),
        many => {
            let names: Vec<&str> = many.iter().map(|(n, _)| *n).collect();
            format!("no such option {name}. (Possible options: {})", names.join(", "))
        }
    }
}

fn jaro_similarity(s1: &[char], s2: &[char]) -> f64 {
    if s1.is_empty() && s2.is_empty() {
        return 1.0;
    } else if s1.is_empty() || s2.is_empty() {
        return 0.0;
    } else if s1.len() == 1 && s2.len() == 1 {
        return if s1[0] == s2[0] { 1.0 } else { 0.0 };
    }
    let search_range = (s1.len().max(s2.len()) / 2).saturating_sub(1);
    let mut s2_consumed = vec![false; s2.len()];
    let mut matches = 0.0;
    let mut transpositions = 0.0;
    let mut s2_match_index = 0;
    for (i, c1) in s1.iter().enumerate() {
        let start = i.saturating_sub(search_range);
        let end = (s2.len() - 1).min(i + search_range);
        for j in start..=end {
            if *c1 != s2[j] || s2_consumed[j] {
                continue;
            }
            s2_consumed[j] = true;
            matches += 1.0;
            if j < s2_match_index {
                transpositions += 1.0;
            }
            s2_match_index = j;
            break;
        }
    }
    if matches == 0.0 {
        0.0
    } else {
        (matches / s1.len() as f64 + matches / s2.len() as f64 + (matches - transpositions) / matches) / 3.0
    }
}

/// Clikt's `jaroWinklerSimilarity`: no cap on the prefix length.
fn jaro_winkler_similarity(s1: &str, s2: &str) -> f64 {
    let (a, b): (Vec<char>, Vec<char>) = (s1.chars().collect(), s2.chars().collect());
    let prefix_length = a.iter().zip(&b).take_while(|(x, y)| x == y).count() as f64;
    let jaro = jaro_similarity(&a, &b);
    (jaro + 0.1 * prefix_length * (1.0 - jaro)).min(1.0)
}

/// Expands `@file` tokens (`@@x` is the literal `@x`), relative to `working_dir`.
pub fn expand_argument_files(tokens: &[String], working_dir: &Path) -> Result<Vec<String>, String> {
    let mut expanded = Vec::new();
    for token in tokens {
        match token.strip_prefix('@') {
            Some(rest) if rest.starts_with('@') => expanded.push(rest.to_owned()),
            Some(file) => {
                let text = std::fs::read_to_string(working_dir.join(file)).map_err(|_| format!("{file} not found"))?;
                expanded.extend(expand_argument_files(&split_argfile(&text), working_dir)?);
            }
            None => expanded.push(token.clone()),
        }
    }
    Ok(expanded)
}

/// Clikt's argfile tokenizer: whitespace-separated, `'`/`"` quoting, `\` escapes, `#` line comments.
fn split_argfile(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current: Option<String> = None;
    let mut quote: Option<char> = None;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (_, '\\') => {
                if let Some(next) = chars.next() {
                    current.get_or_insert_with(String::new).push(next);
                }
            }
            (Some(_), c) => current.get_or_insert_with(String::new).push(c),
            (None, '\'' | '"') => {
                quote = Some(c);
                current.get_or_insert_with(String::new);
            }
            (None, '#') if current.is_none() => {
                while chars.next_if(|&n| n != '\n').is_some() {}
            }
            (None, c) if c.is_whitespace() => tokens.extend(current.take()),
            (None, c) => current.get_or_insert_with(String::new).push(c),
        }
    }
    tokens.extend(current);
    tokens
}
