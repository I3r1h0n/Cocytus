use std::ops::Range;

use iced::advanced::text::highlighter::{self, Highlighter};
use iced::{Color, Font, Theme};

// Syntax colors

const JSON_KEY: Color =
    Color::from_rgb(0x79 as f32 / 255.0, 0xc0 as f32 / 255.0, 0xff as f32 / 255.0);
const JSON_STRING: Color =
    Color::from_rgb(0xa5 as f32 / 255.0, 0xd6 as f32 / 255.0, 0xa7 as f32 / 255.0);
const JSON_NUMBER: Color =
    Color::from_rgb(0xd2 as f32 / 255.0, 0xa8 as f32 / 255.0, 0xff as f32 / 255.0);
const JSON_BOOL: Color =
    Color::from_rgb(0xff as f32 / 255.0, 0xa6 as f32 / 255.0, 0x57 as f32 / 255.0);
const JSON_NULL: Color =
    Color::from_rgb(0xff as f32 / 255.0, 0x7b as f32 / 255.0, 0x72 as f32 / 255.0);
const JSON_PUNCT: Color =
    Color::from_rgb(0x8b as f32 / 255.0, 0x94 as f32 / 255.0, 0x9e as f32 / 255.0);
const JSON_NORMAL: Color =
    Color::from_rgb(0xc9 as f32 / 255.0, 0xd1 as f32 / 255.0, 0xd9 as f32 / 255.0);

// Highlight token

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonToken {
    Key,
    String,
    Number,
    Boolean,
    Null,
    Punctuation,
    Normal,
}

// Highlighter implementation

pub struct JsonHighlighter {
    current_line: usize,
}

impl Highlighter for JsonHighlighter {
    type Settings = ();
    type Highlight = JsonToken;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, JsonToken)>;

    fn new(_settings: &()) -> Self {
        Self { current_line: 0 }
    }

    fn update(&mut self, _new_settings: &()) {}

    fn change_line(&mut self, line: usize) {
        self.current_line = line;
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        self.current_line += 1;
        tokenize_json_line(line).into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}

/// Map a highlight token to its display format
pub fn format(token: &JsonToken, _theme: &Theme) -> highlighter::Format<Font> {
    let color = match token {
        JsonToken::Key => JSON_KEY,
        JsonToken::String => JSON_STRING,
        JsonToken::Number => JSON_NUMBER,
        JsonToken::Boolean => JSON_BOOL,
        JsonToken::Null => JSON_NULL,
        JsonToken::Punctuation => JSON_PUNCT,
        JsonToken::Normal => JSON_NORMAL,
    };
    highlighter::Format {
        color: Some(color),
        font: None,
    }
}

// Tokenizer

fn tokenize_json_line(line: &str) -> Vec<(Range<usize>, JsonToken)> {
    let mut tokens = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        match bytes[i] {
            b' ' | b'\t' | b'\r' | b'\n' => {
                let start = i;
                while i < len && matches!(bytes[i], b' ' | b'\t' | b'\r' | b'\n') {
                    i += 1;
                }
                tokens.push((start..i, JsonToken::Normal));
            }
            b'"' => {
                let start = i;
                i += 1; // opening quote
                while i < len && bytes[i] != b'"' {
                    if bytes[i] == b'\\' && i + 1 < len {
                        i += 1; // skip escaped char
                    }
                    i += 1;
                }
                if i < len {
                    i += 1; // closing quote
                }

                // Peek ahead: if the next non-space char is ':', this is a key
                let mut peek = i;
                while peek < len && (bytes[peek] == b' ' || bytes[peek] == b'\t') {
                    peek += 1;
                }
                let token = if peek < len && bytes[peek] == b':' {
                    JsonToken::Key
                } else {
                    JsonToken::String
                };
                tokens.push((start..i, token));
            }
            b'{' | b'}' | b'[' | b']' | b',' | b':' => {
                tokens.push((i..i + 1, JsonToken::Punctuation));
                i += 1;
            }
            b't' if bytes[i..].starts_with(b"true") => {
                tokens.push((i..i + 4, JsonToken::Boolean));
                i += 4;
            }
            b'f' if bytes[i..].starts_with(b"false") => {
                tokens.push((i..i + 5, JsonToken::Boolean));
                i += 5;
            }
            b'n' if bytes[i..].starts_with(b"null") => {
                tokens.push((i..i + 4, JsonToken::Null));
                i += 4;
            }
            b'0'..=b'9' | b'-' => {
                let start = i;
                if bytes[i] == b'-' {
                    i += 1;
                }
                while i < len
                    && (bytes[i].is_ascii_digit()
                        || matches!(bytes[i], b'.' | b'e' | b'E' | b'+' | b'-'))
                {
                    i += 1;
                }
                // Only highlight as number if we consumed at least one digit
                if i > start + usize::from(bytes[start] == b'-') {
                    tokens.push((start..i, JsonToken::Number));
                } else {
                    tokens.push((start..i, JsonToken::Normal));
                }
            }
            _ => {
                tokens.push((i..i + 1, JsonToken::Normal));
                i += 1;
            }
        }
    }

    tokens
}
