//! CHAR, WCHAR, STRING and WSTRING: the character literals.
//!
//! The quote is part of the type. STRING and CHAR are single-quoted and hold
//! single-byte characters; WSTRING and WCHAR are double-quoted and hold
//! UTF-16 code units. The standard does not let either borrow the other's
//! quote, and a crate that allowed it would make the two types one.
//!
//! Escapes open with `$` rather than a backslash, which is what keeps a
//! Windows path readable in a controller program:
//!
//! | escape | means |
//! |---|---|
//! | `$$` | a dollar sign |
//! | `$'` `$"` | the closing quote of the respective type |
//! | `$L` `$N` | line feed, new line |
//! | `$P` | form feed |
//! | `$R` | carriage return |
//! | `$T` | tab |
//! | `$hh` / `$hhhh` | a code unit, two hex digits in STRING and four in WSTRING |

use std::fmt;
use std::fmt::Write as _;

use crate::elementary::{ElementaryType, ElementaryValue};
use crate::error::Result;
use crate::error_code::ShlitaCode;

/// The quote a type is written with.
const fn quote(ty: ElementaryType) -> char {
    match ty {
        ElementaryType::Wchar | ElementaryType::WString => '"',
        _ => '\'',
    }
}

/// How many hex digits a numeric escape takes, which is the width of one
/// code unit.
const fn escape_digits(ty: ElementaryType) -> usize {
    match ty {
        ElementaryType::Wchar | ElementaryType::WString => 4,
        _ => 2,
    }
}

/// The largest code point the type can hold.
const fn code_limit(ty: ElementaryType) -> u32 {
    match ty {
        ElementaryType::Wchar | ElementaryType::WString => 0xFFFF,
        _ => 0xFF,
    }
}

/// Read STRING or WSTRING.
pub(crate) fn read_string(ty: ElementaryType, literal: &str) -> Result<ElementaryValue> {
    let body = unquote(ty, literal)?;
    let value = unescape(ty, literal, body)?;
    Ok(ElementaryValue::Text { ty, value })
}

/// Read CHAR or WCHAR.
pub(crate) fn read_char(ty: ElementaryType, literal: &str) -> Result<ElementaryValue> {
    let body = unquote(ty, literal)?;
    let text = unescape(ty, literal, body)?;
    let mut characters = text.chars();
    let (Some(character), None) = (characters.next(), characters.next()) else {
        return Err(ty.err(
            ShlitaCode::NotASingleCharacter,
            literal,
            format!(
                "{ty} holds exactly one character, and this holds {}",
                text.chars().count()
            ),
        ));
    };
    let code = character as u32;
    if code > code_limit(ty) {
        return Err(ty.err(
            ShlitaCode::CharacterOutOfRange,
            literal,
            format!("the code point of {ty} runs to 16#{:X}", code_limit(ty)),
        ));
    }
    Ok(ElementaryValue::Char { ty, code })
}

/// Strip the optional `TYPE#` prefix and the required quotes.
///
/// The prefix is looked for only when the literal does not open with a
/// quote, because a `#` inside a string is a character like any other and
/// `'a#b'` is not a typed literal.
fn unquote(ty: ElementaryType, literal: &str) -> Result<&str> {
    let quote = quote(ty);
    let body = if literal.starts_with(['\'', '"']) {
        literal
    } else {
        match literal.split_once('#') {
            Some((head, rest)) => match ElementaryType::from_name(head) {
                Some(named) if named == ty => rest,
                Some(named) => {
                    return Err(ty.err(
                        ShlitaCode::WrongTypePrefix,
                        literal,
                        format!("the literal is prefixed {named}, and is being read at {ty}"),
                    ))
                }
                None => {
                    return Err(ty.err(
                        ShlitaCode::UnknownTypeName,
                        literal,
                        format!("`{head}` is not the name of an elementary type"),
                    ))
                }
            },
            None => {
                return Err(ty.err(
                    ShlitaCode::MalformedString,
                    literal,
                    format!("a {ty} is written between {quote} quotes"),
                ))
            }
        }
    };

    let other = if quote == '\'' { '"' } else { '\'' };
    if body.starts_with(other) {
        return Err(ty.err(
            ShlitaCode::WrongStringQuote,
            literal,
            format!("{ty} is written between {quote} quotes, not {other} ones"),
        ));
    }
    let inner = body
        .strip_prefix(quote)
        .and_then(|rest| rest.strip_suffix(quote))
        .filter(|_| body.chars().count() >= 2);
    inner.ok_or_else(|| {
        ty.err(
            ShlitaCode::MalformedString,
            literal,
            format!("a {ty} opens and closes with {quote}"),
        )
    })
}

/// Resolve the `$` escapes, and check that every character fits the type.
fn unescape(ty: ElementaryType, literal: &str, body: &str) -> Result<String> {
    let quote = quote(ty);
    let digits = escape_digits(ty);
    let mut out = String::with_capacity(body.len());
    let mut rest = body.chars().peekable();
    while let Some(character) = rest.next() {
        if character != '$' {
            if character as u32 > code_limit(ty) {
                return Err(ty.err(
                    ShlitaCode::CharacterOutOfRange,
                    literal,
                    format!(
                        "`{character}` does not fit {ty}, whose code points run to 16#{:X}",
                        code_limit(ty)
                    ),
                ));
            }
            out.push(character);
            continue;
        }
        let escape = rest.next().ok_or_else(|| {
            ty.err(
                ShlitaCode::MalformedEscape,
                literal,
                "the literal ends in an unfinished escape",
            )
        })?;
        match escape {
            '$' => out.push('$'),
            c if c == quote => out.push(quote),
            'L' | 'l' | 'N' | 'n' => out.push('\u{000A}'),
            'P' | 'p' => out.push('\u{000C}'),
            'R' | 'r' => out.push('\u{000D}'),
            'T' | 't' => out.push('\u{0009}'),
            c if c.is_ascii_hexdigit() => {
                let mut code = String::with_capacity(digits);
                code.push(c);
                for _ in 1..digits {
                    match rest.next() {
                        Some(c) if c.is_ascii_hexdigit() => code.push(c),
                        _ => {
                            return Err(ty.err(
                                ShlitaCode::MalformedEscape,
                                literal,
                                format!("a numeric escape in {ty} is {digits} hex digits"),
                            ))
                        }
                    }
                }
                let code = u32::from_str_radix(&code, 16).expect("hex digits");
                let character = char::from_u32(code).ok_or_else(|| {
                    ty.err(
                        ShlitaCode::CharacterOutOfRange,
                        literal,
                        format!("16#{code:X} is not a character on its own"),
                    )
                })?;
                out.push(character);
            }
            other => {
                return Err(ty.err(
                    ShlitaCode::MalformedEscape,
                    literal,
                    format!("`${other}` is not an escape the standard defines"),
                ))
            }
        }
    }
    Ok(out)
}

/// Write one character back, quoted and escaped.
pub(crate) fn format_char(
    ty: ElementaryType,
    code: u32,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let quote = quote(ty);
    let character = char::from_u32(code).ok_or(fmt::Error)?;
    write!(f, "{}#{quote}", ty.name())?;
    write_escaped(ty, &character.to_string(), f)?;
    f.write_char(quote)
}

/// Write a string back, quoted and escaped.
pub(crate) fn format_string(
    ty: ElementaryType,
    value: &str,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let quote = quote(ty);
    f.write_char(quote)?;
    write_escaped(ty, value, f)?;
    f.write_char(quote)
}

fn write_escaped(ty: ElementaryType, value: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let quote = quote(ty);
    let digits = escape_digits(ty);
    for character in value.chars() {
        match character {
            '$' => f.write_str("$$")?,
            c if c == quote => write!(f, "${quote}")?,
            '\u{000A}' => f.write_str("$N")?,
            '\u{000C}' => f.write_str("$P")?,
            '\u{000D}' => f.write_str("$R")?,
            '\u{0009}' => f.write_str("$T")?,
            c if (c as u32) < 0x20 || c as u32 == 0x7F => {
                write!(f, "${:0digits$X}", c as u32)?;
            }
            c => f.write_char(c)?,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(ty: ElementaryType, literal: &str) -> Result<ElementaryValue> {
        ty.read(literal)
    }

    fn code(ty: ElementaryType, literal: &str) -> ShlitaCode {
        read(ty, literal).unwrap_err().code()
    }

    fn text(ty: ElementaryType, literal: &str) -> String {
        read(ty, literal).unwrap().as_text().unwrap().to_string()
    }

    #[test]
    fn a_string_is_quoted_and_a_wstring_is_double_quoted() {
        assert_eq!(text(ElementaryType::String, "'hello'"), "hello");
        assert_eq!(text(ElementaryType::WString, "\"hello\""), "hello");
        assert_eq!(text(ElementaryType::String, "''"), "");
    }

    /// The quote is the difference between the two types, so borrowing the
    /// other one is named rather than treated as a typo.
    #[test]
    fn neither_string_type_takes_the_others_quote() {
        assert_eq!(
            code(ElementaryType::String, "\"hello\""),
            ShlitaCode::WrongStringQuote
        );
        assert_eq!(
            code(ElementaryType::WString, "'hello'"),
            ShlitaCode::WrongStringQuote
        );
        assert_eq!(
            code(ElementaryType::String, "hello"),
            ShlitaCode::MalformedString
        );
        assert_eq!(
            code(ElementaryType::String, "'hello"),
            ShlitaCode::MalformedString
        );
    }

    #[test]
    fn a_hash_inside_a_string_is_a_character_and_not_a_prefix() {
        assert_eq!(text(ElementaryType::String, "'a#b'"), "a#b");
        assert_eq!(text(ElementaryType::String, "STRING#'a#b'"), "a#b");
    }

    #[test]
    fn the_escapes_are_the_standards() {
        assert_eq!(text(ElementaryType::String, "'a$$b'"), "a$b");
        assert_eq!(text(ElementaryType::String, "'it$'s'"), "it's");
        assert_eq!(text(ElementaryType::String, "'a$Tb'"), "a\tb");
        assert_eq!(text(ElementaryType::String, "'a$Nb'"), "a\nb");
        assert_eq!(text(ElementaryType::String, "'a$0Db'"), "a\rb");
        assert_eq!(text(ElementaryType::WString, "\"a$0041b\""), "aAb");
        assert_eq!(
            code(ElementaryType::String, "'a$Qb'"),
            ShlitaCode::MalformedEscape
        );
        assert_eq!(
            code(ElementaryType::String, "'a$'"),
            ShlitaCode::MalformedEscape
        );
        assert_eq!(
            code(ElementaryType::WString, "\"a$041\""),
            ShlitaCode::MalformedEscape
        );
    }

    /// A STRING holds single-byte characters, which is what the standard
    /// says and what the width of `$hh` implies.
    #[test]
    fn a_string_holds_single_byte_characters_and_a_wstring_holds_code_units() {
        // A code point that fits one byte fits a STRING; one that does not,
        // does not, and the type is where the line is drawn.
        assert_eq!(text(ElementaryType::String, "'grün'"), "grün");
        assert_eq!(
            code(ElementaryType::String, "'20 €'"),
            ShlitaCode::CharacterOutOfRange
        );
        assert_eq!(text(ElementaryType::WString, "\"20 €\""), "20 €");
        assert_eq!(
            code(ElementaryType::WString, "\"$D800\""),
            ShlitaCode::CharacterOutOfRange
        );
    }

    #[test]
    fn a_char_holds_exactly_one_character() {
        assert_eq!(
            read(ElementaryType::Char, "'A'"),
            Ok(ElementaryValue::Char {
                ty: ElementaryType::Char,
                code: 65
            })
        );
        assert_eq!(
            read(ElementaryType::Char, "CHAR#'A'").unwrap().to_string(),
            "CHAR#'A'"
        );
        assert_eq!(
            code(ElementaryType::Char, "'AB'"),
            ShlitaCode::NotASingleCharacter
        );
        assert_eq!(
            code(ElementaryType::Char, "''"),
            ShlitaCode::NotASingleCharacter
        );
        // A CHAR holds one byte, so `ü` fits it and `€` does not.
        assert!(read(ElementaryType::Char, "CHAR#'ü'").is_ok());
        assert_eq!(
            code(ElementaryType::Char, "CHAR#'€'"),
            ShlitaCode::CharacterOutOfRange
        );
        assert_eq!(
            read(ElementaryType::Wchar, "WCHAR#\"ü\"")
                .unwrap()
                .to_string(),
            "WCHAR#\"ü\""
        );
    }

    #[test]
    fn printing_escapes_what_reading_resolved() {
        let value = read(ElementaryType::String, "'a$Tb$$c$'d'").unwrap();
        assert_eq!(value.to_string(), "'a$Tb$$c$'d'");
        let reread = read(ElementaryType::String, &value.to_string()).unwrap();
        assert_eq!(value, reread);
    }
}
