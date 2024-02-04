use std::iter::Enumerate;
use std::num::ParseIntError;
use std::str::Chars;

/// Error type of [unescape](unescape).
#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidSymbol {
        symbol: String,
        index: usize,
        string: String,
    },
    // #[error("invalid escape {escape} at {index} in {string}")]
    InvalidEscape {
        escape: String,
        index: usize,
        string: String,
    },
    // #[error("\\u could not be parsed at {index} in {string}: {source}")]
    InvalidUnicode {
        // #[source]
        source: ParseUnicodeError,
        index: usize,
        string: String,
    },
    MissingOpeningQuote,
    MissingClosingQuote,
}

/// Source error type of [ParseError::InvalidUnicode](ParseError::InvalidUnicode).
#[derive(Debug, PartialEq, Clone)]
pub enum ParseUnicodeError {
    // #[error("could not parse {string} as u32 hex: {source}")]
    ParseHexFailed {
        // #[source]
        source: ParseIntError,
        string: String,
    },
    ParseOctFailed {
        // #[source]
        source: ParseIntError,
        string: String,
    },
    // #[error("could not parse {value} as a unicode char")]
    ParseUnicodeFailed {
        value: u32,
    },
}

/// Parse the provided quoted string.
/// This function was adopted from [snailquote](https://docs.rs/snailquote/latest/snailquote/).
///
/// # Details
///
/// Parses a single or double quoted string and interprets escape sequences such as
/// '\n', '\r', '\'', etc.
///
/// Supports raw strings prefixed with `r` or `R` in which case all escape sequences are ignored.///
///
/// The full set of supported escapes between quotes may be found below:
///
/// | Escape     | Code       | Description                              |
/// |------------|------------|------------------------------------------|
/// | \a         | 0x07       | Bell                                     |
/// | \b         | 0x08       | Backspace                                |
/// | \v         | 0x0B       | Vertical tab                             |
/// | \f         | 0x0C       | Form feed                                |
/// | \n         | 0x0A       | Newline                                  |
/// | \r         | 0x0D       | Carriage return                          |
/// | \t         | 0x09       | Tab                                      |
/// | \\         | 0x5C       | Backslash                                |
/// | \?         | 0x??       | Question mark                            |
/// | \"         | 0x22       | Double quote                             |
/// | \'         | 0x27       | Single quote                             |
/// | \`         | 0x60       | Backtick                                 |
/// | \xDD       | 0xDD       | Unicode character with hex code DD       |
/// | \uDDDD     | 0xDDDD     | Unicode character with hex code DDDD     |
/// | \UDDDDDDDD | 0xDDDDDDDD | Unicode character with hex code DDDDDDDD |
/// | \DDD       | 0DDD       | Unicode character with octal code DDD    |
///
/// # Errors
///
/// The returned result can display a human readable error if the string cannot be parsed as a
/// valid quoted string.
///

pub fn parse_string(s: &str) -> Result<String, ParseError> {
    let mut chars = s.chars().enumerate();
    let res = String::with_capacity(s.len());

    return match chars.next() {
        Some((_, c)) if c == 'r' || c == 'R' => parse_raw_string(&mut chars, res),
        Some((_, c)) if c == '\'' || c == '"' => parse_quoted_string(s, &mut chars, res, c),
        _ => Err(ParseError::MissingOpeningQuote),
    };
}

fn parse_raw_string(chars: &mut Enumerate<Chars>, mut res: String) -> Result<String, ParseError> {
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;

    while let Some((_, c)) = chars.next() {
        let in_quotes = in_single_quotes || in_double_quotes;

        if c == '\\' && in_quotes {
            match chars.next() {
                Some((_, c2)) => {
                    match c2 {
                        '"' => {
                            if in_single_quotes {
                                res.push(c);
                            }
                        }
                        '\'' => {
                            if in_double_quotes {
                                res.push(c);
                            }
                        }
                        _ => {
                            res.push(c);
                        }
                    };
                    res.push(c2);
                    continue;
                }
                _ => {
                    res.push(c);
                    continue;
                }
            };
        } else if c == '\'' {
            if in_double_quotes {
                res.push(c);
                continue;
            }

            in_single_quotes = !in_single_quotes;
            continue;
        } else if c == '"' {
            if in_single_quotes {
                res.push(c);
                continue;
            }

            in_double_quotes = !in_double_quotes;
            continue;
        } else if !in_quotes {
            return Err(ParseError::MissingOpeningQuote);
        }

        res.push(c);
    }

    Ok(res)
}

fn parse_quoted_string(
    s: &str,
    mut chars: &mut Enumerate<Chars>,
    mut res: String,
    quote: char,
) -> Result<String, ParseError> {
    let mut in_single_quotes = quote == '\'';
    let mut in_double_quotes = quote == '"';

    while let Some((idx, c)) = chars.next() {
        let in_quotes = in_single_quotes || in_double_quotes;

        if c == '\\' && in_quotes {
            match chars.next() {
                None => {
                    return Err(ParseError::InvalidEscape {
                        escape: format!("{}", c),
                        index: idx,
                        string: String::from(s),
                    });
                }
                Some((idx, c2)) => {
                    let mut push_escape_character = false;

                    let value = match c2 {
                        'a' => '\u{07}',
                        'b' => '\u{08}',
                        'v' => '\u{0B}',
                        'f' => '\u{0C}',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '\\' => c2,
                        '?' => c2,
                        '\'' => {
                            push_escape_character = in_double_quotes;
                            c2
                        }
                        '"' => {
                            push_escape_character = in_single_quotes;
                            c2
                        }
                        '`' => c2,
                        'x' | 'u' | 'U' => {
                            let length = match c2 {
                                'x' => 2,
                                'u' => 4,
                                'U' => 8,
                                _ => unreachable!(),
                            };

                            parse_unicode_hex(length, &mut chars).map_err(|x| {
                                ParseError::InvalidUnicode {
                                    source: x.clone(),
                                    index: idx,
                                    string: String::from(s),
                                }
                            })?
                        }
                        n if ('0'..='3').contains(&n) => parse_unicode_oct(&n, &mut chars)
                            .map_err(|x| ParseError::InvalidUnicode {
                                source: x.clone(),
                                index: idx,
                                string: String::from(s),
                            })?,
                        _ => {
                            return Err(ParseError::InvalidEscape {
                                escape: format!("{}{}", c, c2),
                                index: idx,
                                string: String::from(s),
                            });
                        }
                    };

                    if push_escape_character {
                        res.push(c);
                    }

                    res.push(value);

                    continue;
                }
            };
        } else if c == '\'' {
            if in_double_quotes {
                res.push(c);
                continue;
            }

            in_single_quotes = !in_single_quotes;
            continue;
        } else if c == '"' {
            if in_single_quotes {
                res.push(c);
                continue;
            }

            in_double_quotes = !in_double_quotes;
            continue;
        } else if !in_quotes {
            return Err(ParseError::MissingOpeningQuote);
        }

        res.push(c);
    }

    // Ensure string has a closing quote
    if in_single_quotes || in_double_quotes {
        return Err(ParseError::MissingClosingQuote);
    }

    Ok(res)
}

fn parse_unicode_hex<I>(length: usize, chars: &mut I) -> Result<char, ParseUnicodeError>
where
    I: Iterator<Item = (usize, char)>,
{
    let unicode_seq: String = chars.take(length).map(|(_, c)| c).collect();

    u32::from_str_radix(&unicode_seq, 16)
        .map_err(|e| ParseUnicodeError::ParseHexFailed {
            source: e,
            string: unicode_seq,
        })
        .and_then(|u| char::from_u32(u).ok_or(ParseUnicodeError::ParseUnicodeFailed { value: u }))
}

fn parse_unicode_oct<I>(first_char: &char, chars: &mut I) -> Result<char, ParseUnicodeError>
where
    I: Iterator<Item = (usize, char)>,
{
    let mut unicode_seq: String = String::with_capacity(3);
    unicode_seq.push(*first_char);
    chars.take(2).for_each(|(_, c)| unicode_seq.push(c));

    u32::from_str_radix(&unicode_seq, 8)
        .map_err(|e| ParseUnicodeError::ParseOctFailed {
            source: e,
            string: unicode_seq,
        })
        .and_then(|u| {
            if u <= 255 {
                char::from_u32(u).ok_or(ParseUnicodeError::ParseUnicodeFailed { value: u })
            } else {
                Err(ParseUnicodeError::ParseUnicodeFailed { value: u })
            }
        })
}

#[cfg(test)]
mod tests {
    use crate::parse::ParseError;
    use crate::parse_string;

    #[test]
    fn single_quotes_interprets_escapes() {
        let tests: Vec<(&str, Result<String, ParseError>)> = vec![
            ("'Hello \\a'", Ok(String::from("Hello \u{07}"))),
            ("'Hello \\b'", Ok(String::from("Hello \u{08}"))),
            ("'Hello \\v'", Ok(String::from("Hello \u{0b}"))),
            ("'Hello \\f'", Ok(String::from("Hello \u{0c}"))),
            ("'Hello \\n'", Ok(String::from("Hello \u{0a}"))),
            ("'Hello \\r'", Ok(String::from("Hello \u{0d}"))),
            ("'Hello \\t'", Ok(String::from("Hello \u{09}"))),
            ("'Hello \\\\'", Ok(String::from("Hello \\"))),
            ("'Hello \\?'", Ok(String::from("Hello ?"))),
            ("'Hello \"'", Ok(String::from("Hello \""))),
            ("'Hello \\''", Ok(String::from("Hello '"))),
            ("'Hello \\`'", Ok(String::from("Hello `"))),
            ("'Hello \\x20'", Ok(String::from("Hello  "))),
            ("'Hello \\u270c'", Ok(String::from("Hello ✌"))),
            ("'Hello \\U0001f431'", Ok(String::from("Hello 🐱"))),
            ("'Hello \\040'", Ok(String::from("Hello  "))),
            (
                "Missing closing quote'",
                Err(ParseError::MissingOpeningQuote),
            ),
            (
                "'Missing closing quote",
                Err(ParseError::MissingClosingQuote),
            ),
            // Testing octal value is out of range
            (
                "'\\440'",
                Err(ParseError::InvalidEscape {
                    escape: String::from("\\4"),
                    index: 2,
                    string: String::from("'\\440'"),
                }),
            ),
        ];

        for (s, expected) in tests {
            let result = parse_string(s);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn double_quotes_interprets_escapes() {
        let tests: Vec<(&str, Result<String, ParseError>)> = vec![
            ("\"Hello \\a\"", Ok(String::from("Hello \u{07}"))),
            ("\"Hello \\b\"", Ok(String::from("Hello \u{08}"))),
            ("\"Hello \\v\"", Ok(String::from("Hello \u{0b}"))),
            ("\"Hello \\f\"", Ok(String::from("Hello \u{0c}"))),
            ("\"Hello \\n\"", Ok(String::from("Hello \u{0a}"))),
            ("\"Hello \\r\"", Ok(String::from("Hello \u{0d}"))),
            ("\"Hello \\t\"", Ok(String::from("Hello \u{09}"))),
            ("\"Hello \\\\\"", Ok(String::from("Hello \\"))),
            ("\"Hello \\?\"", Ok(String::from("Hello ?"))),
            ("\"Hello \\\"\"", Ok(String::from("Hello \""))),
            ("\"Hello \\'\"", Ok(String::from("Hello \\'"))),
            ("\"Hello \\`\"", Ok(String::from("Hello `"))),
            ("\"Hello \\x20 \"", Ok(String::from("Hello   "))),
            ("\"Hello \\x60\"", Ok(String::from("Hello `"))),
            ("\"Hello \\u270c\"", Ok(String::from("Hello ✌"))),
            ("\"Hello \\U0001f431\"", Ok(String::from("Hello 🐱"))),
            ("\"Hello \\040\"", Ok(String::from("Hello  "))),
            (
                "Missing closing quote\"",
                Err(ParseError::MissingOpeningQuote),
            ),
            (
                "\"Missing closing quote",
                Err(ParseError::MissingClosingQuote),
            ),
            // Testing octal value is out of range
            (
                "\"\\440\"",
                Err(ParseError::InvalidEscape {
                    escape: String::from("\\4"),
                    index: 2,
                    string: String::from("\"\\440\""),
                }),
            ),
        ];

        for (s, expected) in tests {
            let result = parse_string(s);
            assert_eq!(result, expected, "Testing {}", s);
        }
    }

    #[test]
    fn raw_string_does_not_interpret_escapes() {
        let tests: Vec<(&str, Result<String, ParseError>)> = vec![
            // Raw string in double quotes
            // r"Hello \a \" ' \' \U0001f431 " => Hello \a " ' \' \U0001f431
            // R"Hello \a \" ' \' \U0001f431 " => Hello \a " ' \' \U0001f431
            (
                "r\"Hello \\a \\\" ' \\' \\U0001f431 \"",
                Ok(String::from("Hello \\a \" ' \\' \\U0001f431 ")),
            ),
            (
                "R\"Hello \\a \\\" ' \\' \\U0001f431 \"",
                Ok(String::from("Hello \\a \" ' \\' \\U0001f431 ")),
            ),
            // Raw string in single quotes
            // r'Hello \a \" " \' \U0001f431 ' => Hello \a \" " ' \U0001f431
            // R'Hello \a \" " \' \U0001f431 ' => Hello \a \" " ' \U0001f431
            (
                "r'Hello \\a \\\" \" \\' \\U0001f431 '",
                Ok(String::from("Hello \\a \\\" \" ' \\U0001f431 ")),
            ),
            (
                "R'Hello \\a \\\" \" \\' \\U0001f431 '",
                Ok(String::from("Hello \\a \\\" \" ' \\U0001f431 ")),
            ),
        ];

        for (s, expected) in tests {
            let result = parse_string(s);
            assert_eq!(result, expected, "Testing {}", s);
        }
    }
}
