use std::{ops::Deref, str::FromStr, sync::LazyLock};

use anyhow::{Error, anyhow};
use fancy_regex::Regex;
use ratatui::{
    style::Style,
    text::{Line, Span, Text},
};
use serde::Deserialize;
use serde_with::{self, DisplayFromStr, serde_as};

use crate::theme::ColourTheme;

pub static RUST_SYNTAX: LazyLock<SyntaxRegex> = LazyLock::new(|| {
    SyntaxRegex::new(
        r"^(fn|cfg|super|let|mut|mod|pub|const|impl|static|for|use|while|match|if|else|break|continue|struct|enum|self)\b",
        r"^[A-Za-z_][A-Za-z0-9_]*",
        r"^(\(|\)|\||\{|\}|\[|\]|;|:|,|<|>|\?|\#)",
        r#"^(r\#\".*\"\#|\".*\"|[0-9]+)"#,
        r"^([A-Z][A-Za-z0-9_]*|str)",
        r"^(==|!=|<=|>=|=|\+|-|\*|/|\.\.|=>)",
        r"^([a-z][a-z_0-9]*)(?=\()",
        r"^(\/\/.*|/\*([\s\S]*?)\*/)",
    )
    .unwrap()
});

#[derive(Debug, Clone)]
pub struct CRegex(Regex);

impl CRegex {
    pub fn new<T>(thing: T) -> anyhow::Result<Self>
    where
        T: TryInto<Regex>,
    {
        Ok(Self(
            thing
                .try_into()
                .map_err(|_e| anyhow!("failed building regex"))?,
        ))
    }
}

impl Deref for CRegex {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for CRegex {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Regex::new(s).map_err(|e| anyhow!("{e}"))?))
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct SyntaxRegex {
    #[serde_as(as = "DisplayFromStr")]
    pub keyword: CRegex,
    #[serde_as(as = "DisplayFromStr")]
    pub identifier: CRegex,
    #[serde_as(as = "DisplayFromStr")]
    pub function: CRegex,
    #[serde_as(as = "DisplayFromStr")]
    pub delimiters: CRegex,
    #[serde_as(as = "DisplayFromStr")]
    pub literal: CRegex,
    #[serde_as(as = "DisplayFromStr")]
    pub types: CRegex,
    #[serde_as(as = "DisplayFromStr")]
    pub comment: CRegex,
    #[serde_as(as = "DisplayFromStr")]
    pub extra: CRegex,
}

pub fn colour_text<'a>(text: &'a str, theme: &ColourTheme, syntax: &SyntaxRegex) -> Text<'a> {
    let styled_lines: Vec<Line<'a>> = text
        .lines()
        .map(|line| {
            let line_spans = syntax
                .parse(line)
                .iter()
                .map(|(val, kind)| {
                    Span::raw(*val).style(match kind {
                        SyntaxKind::Keyword => Style::new().fg(theme.keyword.into()),
                        SyntaxKind::Identifier => Style::new().fg(theme.ident.into()),
                        SyntaxKind::Delimiter | SyntaxKind::Whitespace => {
                            Style::new().fg(theme.delim.into())
                        }
                        SyntaxKind::Type => Style::new().fg(theme.types.into()),
                        SyntaxKind::Extra | SyntaxKind::Unknown => {
                            Style::new().fg(theme.extra.into())
                        }
                        SyntaxKind::Literal => Style::new().fg(theme.lit.into()),
                        SyntaxKind::Function => Style::new().fg(theme.function.into()),
                        SyntaxKind::Comment => Style::new().fg(theme.comment.into()),
                    })
                })
                .collect::<Vec<Span<'a>>>();
            Line::from(line_spans)
        })
        .collect();
    Text::from(styled_lines)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SyntaxKind {
    Keyword,
    Identifier,
    Delimiter,
    Literal,
    Function,
    Type,
    Extra,
    Whitespace,
    Comment,
    Unknown,
}

impl SyntaxRegex {
    pub fn new(
        keyword: &str,
        identifier: &str,
        delimiters: &str,
        literal: &str,
        types: &str,
        extra: &str,
        function: &str,
        comment: &str,
    ) -> Result<Self, Error> {
        Ok(Self {
            keyword: CRegex::new(keyword)?,
            identifier: CRegex::new(identifier)?,
            delimiters: CRegex::new(delimiters)?,
            literal: CRegex::new(literal)?,
            types: CRegex::new(types)?,
            extra: CRegex::new(extra)?,
            function: CRegex::new(function)?,
            comment: CRegex::new(comment)?,
        })
    }
    pub fn parse<'a>(&self, text: &'a str) -> Vec<(&'a str, SyntaxKind)> {
        let mut tokens = Vec::new();
        let mut input = text;

        while !input.is_empty() {
            if let Some(non_ws) = input.find(|c: char| !c.is_whitespace()) {
                if non_ws > 0 {
                    let (ws, rest) = input.split_at(non_ws);
                    tokens.push((ws, SyntaxKind::Whitespace));
                    input = rest;
                    continue;
                }
            } else {
                tokens.push((input, SyntaxKind::Whitespace));
                break;
            }

            let mut matched_any = false;

            macro_rules! try_rule {
                ($regex:expr, $kind:expr) => {{
                    if let Ok(Some(m)) = $regex.find(input) {
                        if m.start() == 0 {
                            let end = m.end();

                            if end == 0 {
                                let ch = input.chars().next().unwrap_or_default();
                                let len = ch.len_utf8();
                                let (tok, rest) = input.split_at(len);
                                tokens.push((tok, SyntaxKind::Unknown));
                                input = rest;
                                matched_any = true;
                                continue;
                            }

                            let (tok, rest) = input.split_at(end);
                            tokens.push((tok, $kind));
                            input = rest;
                            matched_any = true;
                            continue;
                        }
                    }
                }};
            }

            try_rule!(self.comment, SyntaxKind::Comment);
            try_rule!(self.literal, SyntaxKind::Literal);
            try_rule!(self.keyword, SyntaxKind::Keyword);
            try_rule!(self.function, SyntaxKind::Function);
            try_rule!(self.types, SyntaxKind::Type);
            try_rule!(self.identifier, SyntaxKind::Identifier);
            try_rule!(self.extra, SyntaxKind::Extra);
            try_rule!(self.delimiters, SyntaxKind::Delimiter);

            if !matched_any {
                let ch = input.chars().next().unwrap_or_default();
                let len = ch.len_utf8();
                let (tok, rest) = input.split_at(len);
                tokens.push((tok, SyntaxKind::Unknown));
                input = rest;
            }
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_SYNTAX: LazyLock<SyntaxRegex> = LazyLock::new(|| {
        SyntaxRegex::new(
            r#"^(fn|let|mut|pub|const|static)\b"#,
            r#"^[A-Za-z_][A-Za-z0-9_]*"#,
            r#"^(\(|\)|\||\{|\}|\[|\]|;|:|,|<|>)"#,
            r#"^(\"\")"#,
            r#"^[A-Z][A-Za-z0-9_]*"#,
            r#"^(==|!=|<=|>=|=|\+|-|\*|/)"#,
            r#"^[98]"#,
            r#"^thisshouldneverbematched"#,
        )
        .unwrap()
    });

    fn non_ws(tokens: Vec<(&str, SyntaxKind)>) -> Vec<(&str, SyntaxKind)> {
        tokens
            .into_iter()
            .filter(|(t, _)| !t.trim().is_empty())
            .collect()
    }

    #[test]
    fn test_keywords() {
        let input = "fn let mut pub const static";
        let tokens = non_ws(TEST_SYNTAX.parse(input));

        for (_, kind) in tokens {
            assert_eq!(kind, SyntaxKind::Keyword);
        }
    }

    #[test]
    fn test_split() {
        let input = "fn let thing = 3;";
        let tokens = TEST_SYNTAX.parse(input);

        let dec_tok: String = tokens.iter().map(|(text, _)| text.to_owned()).collect();
        assert_eq!(dec_tok, input);
    }

    #[test]
    fn test_identifiers() {
        let input = "hello world foo_bar x1 _hidden";
        let tokens = non_ws(TEST_SYNTAX.parse(input));

        for (_, kind) in tokens {
            assert_eq!(kind, SyntaxKind::Identifier);
        }
    }

    #[test]
    fn test_delimiters() {
        let input = "( ) { } [ ] ; : , < > |";
        let tokens = non_ws(TEST_SYNTAX.parse(input));

        for (_, kind) in tokens {
            assert_eq!(kind, SyntaxKind::Delimiter);
        }
    }

    #[test]
    fn test_types() {
        let input = "String MyType HTTPResponse";
        let tokens = non_ws(TEST_SYNTAX.parse(input));

        for (_, kind) in tokens {
            assert_eq!(kind, SyntaxKind::Type);
        }
    }

    #[test]
    fn test_extra() {
        let input = "+ - * / = == != <=";
        let tokens = non_ws(TEST_SYNTAX.parse(input));

        for (_, kind) in tokens {
            assert_eq!(kind, SyntaxKind::Extra);
        }
    }

    #[test]
    fn test_full_snippet() {
        let input = r#"pub fn greet(name: String) { let msg = name + 1; }"#;
        let tokens = TEST_SYNTAX.parse(input);

        assert!(tokens.contains(&("pub", SyntaxKind::Keyword)));
        assert!(tokens.contains(&("fn", SyntaxKind::Keyword)));
        assert!(tokens.contains(&("greet", SyntaxKind::Identifier)));
        assert!(tokens.contains(&("name", SyntaxKind::Identifier)));
        assert!(tokens.contains(&("String", SyntaxKind::Type)));
        assert!(tokens.contains(&("=", SyntaxKind::Extra)));
        assert!(tokens.contains(&("+", SyntaxKind::Extra)));
        assert!(tokens.contains(&("{", SyntaxKind::Delimiter)));
        assert!(tokens.contains(&("}", SyntaxKind::Delimiter)));
    }

    #[test]
    fn test_unknown_tokens() {
        let input = "@$?";
        let tokens = non_ws(TEST_SYNTAX.parse(input));

        for (_, kind) in tokens {
            assert_eq!(kind, SyntaxKind::Unknown);
        }
    }
}
