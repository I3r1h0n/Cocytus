use std::ops::Range;

use iced::advanced::text::highlighter::{self, Highlighter};
use iced::{Color, Font, Theme};

// Syntax colors

const CPP_KEYWORD: Color =
    Color::from_rgb(0xff as f32 / 255.0, 0x7b as f32 / 255.0, 0x72 as f32 / 255.0);
const CPP_TYPE: Color =
    Color::from_rgb(0x79 as f32 / 255.0, 0xc0 as f32 / 255.0, 0xff as f32 / 255.0);
const CPP_COMMENT: Color =
    Color::from_rgb(0x8b as f32 / 255.0, 0x94 as f32 / 255.0, 0x9e as f32 / 255.0);
const CPP_NUMBER: Color =
    Color::from_rgb(0xd2 as f32 / 255.0, 0xa8 as f32 / 255.0, 0xff as f32 / 255.0);
const CPP_PUNCT: Color =
    Color::from_rgb(0x6e as f32 / 255.0, 0x76 as f32 / 255.0, 0x81 as f32 / 255.0);
const CPP_NORMAL: Color =
    Color::from_rgb(0xc9 as f32 / 255.0, 0xd1 as f32 / 255.0, 0xd9 as f32 / 255.0);

// Token types

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CppToken {
    Keyword,
    Type,
    Comment,
    Number,
    Punctuation,
    Normal,
}

// Highlighter

pub struct CppHighlighter {
    current_line: usize,
}

impl Highlighter for CppHighlighter {
    type Settings = ();
    type Highlight = CppToken;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, CppToken)>;

    fn new(_settings: &()) -> Self {
        Self { current_line: 0 }
    }

    fn update(&mut self, _new_settings: &()) {}

    fn change_line(&mut self, line: usize) {
        self.current_line = line;
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        self.current_line += 1;
        tokenize_cpp_line(line).into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}

/// Map a highlight token to its display format
pub fn format(token: &CppToken, _theme: &Theme) -> highlighter::Format<Font> {
    let color = match token {
        CppToken::Keyword => CPP_KEYWORD,
        CppToken::Type => CPP_TYPE,
        CppToken::Comment => CPP_COMMENT,
        CppToken::Number => CPP_NUMBER,
        CppToken::Punctuation => CPP_PUNCT,
        CppToken::Normal => CPP_NORMAL,
    };
    highlighter::Format {
        color: Some(color),
        font: None,
    }
}

// Keyword / type classification

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "struct"
            | "enum"
            | "union"
            | "class"
            | "typedef"
            | "const"
            | "volatile"
            | "virtual"
            | "static"
            | "inline"
            | "__stdcall"
            | "__cdecl"
            | "__fastcall"
            | "__thiscall"
            | "__vectorcall"
            | "__clrcall"
    )
}

fn is_type(word: &str) -> bool {
    matches!(
        word,
        "void"
            | "char"
            | "short"
            | "int"
            | "long"
            | "float"
            | "double"
            | "bool"
            | "signed"
            | "unsigned"
            | "wchar_t"
            | "int8_t"
            | "int16_t"
            | "int32_t"
            | "int64_t"
            | "uint8_t"
            | "uint16_t"
            | "uint32_t"
            | "uint64_t"
            | "size_t"
            | "uintptr_t"
            | "intptr_t"
            | "ptrdiff_t"
            | "BYTE"
            | "WORD"
            | "DWORD"
            | "QWORD"
            | "BOOL"
            | "BOOLEAN"
            | "CHAR"
            | "SHORT"
            | "LONG"
            | "LONGLONG"
            | "UCHAR"
            | "USHORT"
            | "ULONG"
            | "ULONGLONG"
            | "HANDLE"
            | "PVOID"
            | "LPVOID"
            | "HRESULT"
            | "NTSTATUS"
            | "VOID"
            | "INT"
            | "UINT"
    )
}

// Tokenizer

fn tokenize_cpp_line(line: &str) -> Vec<(Range<usize>, CppToken)> {
    let mut tokens = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Whitespace
        if bytes[i].is_ascii_whitespace() {
            let start = i;
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            tokens.push((start..i, CppToken::Normal));
            continue;
        }

        // Line comment: // ...
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            tokens.push((i..len, CppToken::Comment));
            break;
        }

        // Block comment: /* ... */
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            let start = i;
            i += 2;
            while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            if i + 1 < len {
                i += 2;
            } else {
                i = len;
            }
            tokens.push((start..i, CppToken::Comment));
            continue;
        }

        // Hex number: 0x...
        if bytes[i] == b'0' && i + 1 < len && (bytes[i + 1] == b'x' || bytes[i + 1] == b'X') {
            let start = i;
            i += 2;
            while i < len && bytes[i].is_ascii_hexdigit() {
                i += 1;
            }
            tokens.push((start..i, CppToken::Number));
            continue;
        }

        // Negative hex number: -0x...
        if bytes[i] == b'-'
            && i + 2 < len
            && bytes[i + 1] == b'0'
            && (bytes[i + 2] == b'x' || bytes[i + 2] == b'X')
        {
            let start = i;
            i += 3;
            while i < len && bytes[i].is_ascii_hexdigit() {
                i += 1;
            }
            tokens.push((start..i, CppToken::Number));
            continue;
        }

        // Decimal number
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < len && bytes[i].is_ascii_digit() {
                i += 1;
            }
            tokens.push((start..i, CppToken::Number));
            continue;
        }

        // Punctuation
        if matches!(
            bytes[i],
            b'{' | b'}'
                | b'(' | b')'
                | b'[' | b']'
                | b';' | b',' | b':'
                | b'*' | b'&'
                | b'=' | b'<' | b'>'
        ) {
            tokens.push((i..i + 1, CppToken::Punctuation));
            i += 1;
            continue;
        }

        // Words: identifiers, keywords, types
        if bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' {
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &line[start..i];
            let token = if is_keyword(word) {
                CppToken::Keyword
            } else if is_type(word) {
                CppToken::Type
            } else {
                CppToken::Normal
            };
            tokens.push((start..i, token));
            continue;
        }

        // Anything else
        tokens.push((i..i + 1, CppToken::Normal));
        i += 1;
    }

    tokens
}
