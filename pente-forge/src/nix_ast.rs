//! A typed Nix AST.
//!
//! ★ `format!()` of Nix SYNTAX is banned fleet-wide (theory/NIX-AST.md), and the
//! failure it prevents is silent: an unbalanced brace or a missing semicolon
//! ships as a file that PARSES to the wrong thing rather than failing. So a
//! renderer builds a value and prints it once, here.
//!
//! The distinction the ban actually draws: writing a `String` through a
//! `Display`-family `write!` is the sanctioned typed path; interpolating
//! caller-supplied fragments into a syntax template is not. Every brace,
//! semicolon and quote in the output below is emitted by this module and by
//! nothing else, so there is exactly one place to get it wrong.

use std::fmt::Write as _;

/// A Nix value. Deliberately small: this emits palettes, not programs.
#[derive(Debug, Clone, PartialEq)]
pub enum NixExpr {
    Str(String),
    Int(i64),
    Bool(bool),
    /// Ordered — attribute order is stable output, and stable output is what
    /// makes a byte-differential a meaningful check rather than noise.
    AttrSet(Vec<(String, NixExpr)>),
    List(Vec<NixExpr>),
}

impl NixExpr {
    /// A Nix string literal with the four escapes that matter.
    ///
    /// Palette values are hex, so none of these can fire today — which is
    /// exactly why it is written now rather than after the first theme whose
    /// title contains an apostrophe.
    fn write_str(out: &mut String, s: &str) {
        out.push('"');
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '$' => out.push_str("\\$"),
                other => out.push(other),
            }
        }
        out.push('"');
    }

    /// A bare attribute name if it is one, a quoted one otherwise.
    fn write_key(out: &mut String, k: &str) {
        let bare = !k.is_empty()
            && k.chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && k.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        if bare {
            out.push_str(k);
        } else {
            Self::write_str(out, k);
        }
    }

    fn write(&self, out: &mut String, indent: usize) {
        let pad = "  ".repeat(indent);
        match self {
            Self::Str(s) => Self::write_str(out, s),
            Self::Int(i) => {
                let _ = write!(out, "{i}");
            }
            Self::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Self::List(items) => {
                if items.is_empty() {
                    out.push_str("[ ]");
                    return;
                }
                out.push_str("[\n");
                for it in items {
                    out.push_str(&"  ".repeat(indent + 1));
                    it.write(out, indent + 1);
                    out.push('\n');
                }
                out.push_str(&pad);
                out.push(']');
            }
            Self::AttrSet(entries) => {
                if entries.is_empty() {
                    out.push_str("{ }");
                    return;
                }
                out.push_str("{\n");
                for (k, v) in entries {
                    out.push_str(&"  ".repeat(indent + 1));
                    Self::write_key(out, k);
                    out.push_str(" = ");
                    v.write(out, indent + 1);
                    out.push_str(";\n");
                }
                out.push_str(&pad);
                out.push('}');
            }
        }
    }
}

/// A complete `.nix` file: provenance comments plus one expression.
#[derive(Debug, Clone)]
pub struct NixFile {
    pub header: Vec<String>,
    pub expr: NixExpr,
}

impl NixFile {
    #[must_use]
    pub fn new(header: impl IntoIterator<Item = impl Into<String>>, expr: NixExpr) -> Self {
        Self {
            header: header.into_iter().map(Into::into).collect(),
            expr,
        }
    }

    /// Render, with a trailing newline.
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::new();
        for line in &self.header {
            if line.is_empty() {
                out.push_str("#\n");
            } else {
                let _ = writeln!(out, "# {line}");
            }
        }
        self.expr.write(&mut out, 0);
        out.push('\n');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_attrset_renders_with_stable_order_and_semicolons() {
        let f = NixFile::new(
            ["Generated — DO NOT EDIT"],
            NixExpr::AttrSet(vec![
                ("base00".into(), NixExpr::Str("2e3440".into())),
                ("size".into(), NixExpr::Int(13)),
                ("dark".into(), NixExpr::Bool(true)),
            ]),
        );
        assert_eq!(
            f.render(),
            "# Generated — DO NOT EDIT\n{\n  base00 = \"2e3440\";\n  size = 13;\n  dark = true;\n}\n"
        );
    }

    /// The escapes exist before a theme needs them, not after.
    #[test]
    fn strings_escape_the_four_characters_that_matter() {
        let mut s = String::new();
        NixExpr::write_str(&mut s, r#"a"b\c$d"#);
        assert_eq!(s, r#""a\"b\\c\$d""#);
    }

    /// `base0a` is bare; a key needing quotes gets them.
    #[test]
    fn keys_are_quoted_only_when_they_must_be() {
        let mut s = String::new();
        NixExpr::write_key(&mut s, "base0a");
        NixExpr::write_key(&mut s, "with space");
        assert_eq!(s, "base0a\"with space\"");
    }

    #[test]
    fn empty_containers_do_not_emit_a_dangling_newline() {
        let mut s = String::new();
        NixExpr::AttrSet(vec![]).write(&mut s, 0);
        NixExpr::List(vec![]).write(&mut s, 0);
        assert_eq!(s, "{ }[ ]");
    }

    #[test]
    fn nesting_indents_and_closes_correctly() {
        let e = NixExpr::AttrSet(vec![(
            "outer".into(),
            NixExpr::AttrSet(vec![("inner".into(), NixExpr::Str("v".into()))]),
        )]);
        let mut s = String::new();
        e.write(&mut s, 0);
        assert_eq!(s, "{\n  outer = {\n    inner = \"v\";\n  };\n}");
    }
}
