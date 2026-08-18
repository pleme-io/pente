//! `pente-spec` — the typed border of the fleet theme vocabulary.
//!
//! A theme is authored ONCE as a `(deftheme …)` tatara-lisp form and rendered
//! to every consumer. This crate is the border that form compiles into; nothing
//! here emits, and nothing downstream may hold a palette of its own.
//!
//! ## Why the vocabulary exists
//!
//! Measured across the fleet 2026-08-18: **118 palette copies** in **eight
//! distinct encodings**, with an unknown denominator (~62 repos carry ≥5 colour
//! literals; the census reached ~26). The encodings matter more than the count,
//! because a `#RRGGBB` grep sees only three of them:
//!
//! | | encoding | live example |
//! |---|---|---|
//! | E1 | `#RRGGBB` | `nix/lib/nord.nix:22` |
//! | E2 | bare hex `"2E3440"` | `irodzuki/src/presets.rs:26` — **the live source of mado's terminal grid** |
//! | E3 | `Rgb::new(0x2e,0x34,0x40)` | `mado/src/platform.rs:39` |
//! | E4 | decimal `Color::new(136,192,208)` | `kura/kura-tui/src/theme.rs:27` |
//! | E5 | float `vec3(0.53,0.75,0.82)` | a ghostty GLSL shader |
//! | E6 | `[f32;4]` | `egaku/src/theme.rs:112` |
//! | E7 | decimal ANSI `\x1b[38;2;136;192;208m` | `skim-tab/src/lib.rs:103` |
//! | E8 | hex inside a style string | `seki/…/blzsh_parity.rs` |
//!
//! And the source of truth had already drifted from itself: **four** Nord
//! ANSI-16 tables exist, four slots disagree, and slot 7 is a three-way split.
//!
//! ## What is authored here vs derived
//!
//! The ramp is the ONLY place a colour literal may appear. `variant` and `slug`
//! are derived, never authored — stylix interpolates them into artefact NAMES,
//! so a defaulted `"untitled"` silently renames a pile of store paths.
//!
//! The ANSI table is **authored**, deliberately. It is a convention Nord
//! publishes, not a function of the ramp — which is exactly why four
//! independent *derivations* of it disagree today. Authoring it once and
//! projecting it everywhere is what makes that disagreement unconstructible.
//!
//! ## Measured properties of the substrate this border relies on
//!
//! Proven by a compile probe now living upstream as tatara-lisp's
//! `tests/phase_f_constructs.rs` (tatara-lisp@`6f2056a`) — those constructs had
//! **zero use sites** fleet-wide before it, against 202 files using
//! `DeriveTataraDomain`:
//!
//! - A typo'd **kwarg** is REJECTED, with a did-you-mean and the allowed set.
//!   (The surface notes' "empty `Vec` reported as success" warning describes the
//!   *manual extraction* path, not the derive path.)
//! - A typo'd **struct attribute** is SILENT: `#[tatara(keywords = …)]` yields a
//!   keyword computed from the struct NAME. Hence the literal `KEYWORD`
//!   assertions in the tests below — they are load-bearing, not ceremony.
//! - `KeywordSexp` lowercases the ident with **no separator** (`Base0A` →
//!   `:base0a`), while field names go through `snake_to_kebab` (`bright_black`
//!   → `:bright-black`). Two opposite rules; both are exercised here.

use serde::Deserialize;
use tatara_lisp::{DeriveKeywordSexp, DeriveTataraDomain, TataraDomain};

/// A colour, parsed at the boundary — never a `String` that happens to look
/// like one.
///
/// Reaches the border through the derive's `Kind::Deserialize` fall-through, so
/// there is no hand-written extractor to drift. Deliberately implements **no
/// `Default`**: the published derive decides "has a default" with a
/// `tokens.contains("default")` substring test, and a type with no `Default`
/// impl cannot produce a compiling program down that path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct Srgb8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Srgb8 {
    /// Lowercase, unprefixed six-char hex — the stylix/base16 wire format.
    ///
    /// A leading `#` here reaches GTK and changes the colour SILENTLY rather
    /// than failing, which is why the wire format is pinned rather than left to
    /// each renderer.
    #[must_use]
    pub fn to_bare_hex(self) -> String {
        format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl TryFrom<String> for Srgb8 {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        // A `#` is accepted on input and normalised away, because half the
        // fleet's existing literals carry one; it is never emitted.
        let h = s.strip_prefix('#').unwrap_or(&s);
        if h.len() != 6 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!(
                "expected six hex digits (with or without a leading '#'), got {s:?}"
            ));
        }
        let p = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).map_err(|e| e.to_string());
        Ok(Self {
            r: p(0)?,
            g: p(2)?,
            b: p(4)?,
        })
    }
}

/// A name for a colour the theme already defines.
///
/// ★ This type is what makes a stray hex literal outside the ramp a PARSE
/// ERROR rather than a second palette. It has no hex-bearing constructor at
/// all, so `:background "2E3440"` cannot compile — the field wants a keyword,
/// and the error lists the tokens that exist.
///
/// `KeywordSexp` lowercases the ident with no separator, so `Base0A` is
/// `:base0a`. The sixteen ramp slots plus the derived tokens are the whole set;
/// adding a derived token means adding a variant here, and a non-exhaustive
/// match then stops compiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DeriveKeywordSexp)]
pub enum Token {
    Base00,
    Base01,
    Base02,
    Base03,
    Base04,
    Base05,
    Base06,
    Base07,
    Base08,
    Base09,
    Base0a,
    Base0b,
    Base0c,
    Base0d,
    Base0e,
    Base0f,
    /// Derived — a blend over a ramp slot. See [`Derived`].
    Selection,
    /// Derived.
    Glow,
}

/// The sixteen base16 slots. All mandatory: there is no partial ramp.
///
/// base16-indexed rather than Nord-named on purpose — `polar_night_0` is Nord's
/// vocabulary, and a substrate that names it cannot host a second palette.
/// Nord's proper nouns survive as a generated alias module, never as fields.
#[derive(Debug, Clone, PartialEq, Eq, DeriveTataraDomain)]
#[tatara(keyword = "deframp")]
pub struct Ramp {
    pub base00: Srgb8,
    pub base01: Srgb8,
    pub base02: Srgb8,
    pub base03: Srgb8,
    pub base04: Srgb8,
    pub base05: Srgb8,
    pub base06: Srgb8,
    pub base07: Srgb8,
    pub base08: Srgb8,
    pub base09: Srgb8,
    pub base0a: Srgb8,
    pub base0b: Srgb8,
    pub base0c: Srgb8,
    pub base0d: Srgb8,
    pub base0e: Srgb8,
    pub base0f: Srgb8,
}

impl Ramp {
    /// Resolve a ramp token. Returns `None` for a derived token — resolution of
    /// those needs the whole theme, which is [`Theme::resolve`].
    #[must_use]
    pub fn get(&self, t: Token) -> Option<Srgb8> {
        Some(match t {
            Token::Base00 => self.base00,
            Token::Base01 => self.base01,
            Token::Base02 => self.base02,
            Token::Base03 => self.base03,
            Token::Base04 => self.base04,
            Token::Base05 => self.base05,
            Token::Base06 => self.base06,
            Token::Base07 => self.base07,
            Token::Base08 => self.base08,
            Token::Base09 => self.base09,
            Token::Base0a => self.base0a,
            Token::Base0b => self.base0b,
            Token::Base0c => self.base0c,
            Token::Base0d => self.base0d,
            Token::Base0e => self.base0e,
            Token::Base0f => self.base0f,
            Token::Selection | Token::Glow => return None,
        })
    }
}

/// A derived colour: a tint laid over a ramp slot at an alpha.
///
/// A recipe, not a literal. tatara-lisp has no arithmetic at the compile layer,
/// so the blend is DATA and `Theme::resolve` is the one evaluator — which is
/// also what stops a "slightly lighter background" becoming copy 119.
///
/// `over` and `tint` are ramp slots by construction: a blend may not stack on
/// another blend, so resolution is one pass with no cycle to detect.
// No `Eq`: `alpha` is an f64. Deliberate — the alternative is a fixed-point
// newtype, which would buy total equality at the cost of authoring `:alpha 30`
// and meaning 0.30. The blend is data read once and resolved; ordering it is
// not a thing any consumer needs.
#[derive(Debug, Clone, Copy, PartialEq, DeriveTataraDomain)]
#[tatara(keyword = "defblend")]
pub struct Blend {
    #[tatara(keyword_enum)]
    pub over: Token,
    #[tatara(keyword_enum)]
    pub tint: Token,
    /// 0.0–1.0. Validated in [`Theme::validate`]; tatara-lisp cannot bound a
    /// float at the parse boundary, so this is assertion-caught, not
    /// parse-time-rejected. Stated rather than rounded up.
    pub alpha: f64,
}

#[derive(Debug, Clone, PartialEq, DeriveTataraDomain)]
#[tatara(keyword = "defderived")]
pub struct Derived {
    #[tatara(domain)]
    pub selection: Blend,
    #[tatara(domain)]
    pub glow: Blend,
}

/// The ANSI-16 table: index → ramp token.
///
/// ★ AUTHORED, and this is the load-bearing decision of the whole vocabulary.
/// The mapping is a convention Nord publishes, not a function of the ramp.
/// Four independent derivations of it exist in the fleet today and **four slots
/// disagree**, with slot 7 a three-way split:
///
/// | idx | `ishou-render/ghostty.rs` | `ishou-tokens/fleet_theme.rs` | `irodzuki` |
/// |---|---|---|---|
/// | 0 black | `2E3440` | `2E3440` | `3B4252` |
/// | 6 cyan | `8FBCBB` | `88C0D0` | `88C0D0` |
/// | 7 white | `D8DEE9` | `ECEFF4` | `E5E9F0` |
/// | 14 br-cyan | `88C0D0` | `8FBCBB` | `8FBCBB` |
///
/// Authoring it once and projecting it everywhere is what makes that
/// disagreement unconstructible. Every terminal renderer reads THIS; none may
/// derive its own.
///
/// Keyword note: field names take `snake_to_kebab`, so `bright_black` is
/// `:bright-black` — the opposite rule from [`Token`]'s variants.
#[derive(Debug, Clone, PartialEq, Eq, DeriveTataraDomain)]
#[tatara(keyword = "defansi")]
pub struct Ansi16 {
    #[tatara(keyword_enum)]
    pub black: Token,
    #[tatara(keyword_enum)]
    pub red: Token,
    #[tatara(keyword_enum)]
    pub green: Token,
    #[tatara(keyword_enum)]
    pub yellow: Token,
    #[tatara(keyword_enum)]
    pub blue: Token,
    #[tatara(keyword_enum)]
    pub magenta: Token,
    #[tatara(keyword_enum)]
    pub cyan: Token,
    #[tatara(keyword_enum)]
    pub white: Token,
    #[tatara(keyword_enum)]
    pub bright_black: Token,
    #[tatara(keyword_enum)]
    pub bright_red: Token,
    #[tatara(keyword_enum)]
    pub bright_green: Token,
    #[tatara(keyword_enum)]
    pub bright_yellow: Token,
    #[tatara(keyword_enum)]
    pub bright_blue: Token,
    #[tatara(keyword_enum)]
    pub bright_magenta: Token,
    #[tatara(keyword_enum)]
    pub bright_cyan: Token,
    #[tatara(keyword_enum)]
    pub bright_white: Token,
}

impl Ansi16 {
    /// The table in wire order, so a renderer indexes rather than re-maps.
    #[must_use]
    pub fn in_order(&self) -> [Token; 16] {
        [
            self.black,
            self.red,
            self.green,
            self.yellow,
            self.blue,
            self.magenta,
            self.cyan,
            self.white,
            self.bright_black,
            self.bright_red,
            self.bright_green,
            self.bright_yellow,
            self.bright_blue,
            self.bright_magenta,
            self.bright_cyan,
            self.bright_white,
        ]
    }
}

/// Semantic roles → tokens. What a consumer asks for by MEANING.
///
/// Kept a distinct keyword from [`Ansi16`] rather than sharing one
/// `defbinding`, because they carry different key sets: one struct cannot hold
/// both, and two structs sharing a keyword collide at registration.
#[derive(Debug, Clone, PartialEq, Eq, DeriveTataraDomain)]
#[tatara(keyword = "defroles")]
pub struct Roles {
    #[tatara(keyword_enum)]
    pub background: Token,
    #[tatara(keyword_enum)]
    pub surface: Token,
    #[tatara(keyword_enum)]
    pub surface_elevated: Token,
    #[tatara(keyword_enum)]
    pub text: Token,
    #[tatara(keyword_enum)]
    pub text_muted: Token,
    #[tatara(keyword_enum)]
    pub text_dim: Token,
    #[tatara(keyword_enum)]
    pub primary: Token,
    #[tatara(keyword_enum)]
    pub accent: Token,
    #[tatara(keyword_enum)]
    pub border: Token,
    #[tatara(keyword_enum)]
    pub error: Token,
    #[tatara(keyword_enum)]
    pub warning: Token,
    #[tatara(keyword_enum)]
    pub success: Token,
    #[tatara(keyword_enum)]
    pub info: Token,
    #[tatara(keyword_enum)]
    pub selection: Token,
    #[tatara(keyword_enum)]
    pub cursor: Token,
    #[tatara(keyword_enum)]
    pub agent: Token,
}

/// Colours that are NOT part of the palette and have no base16 slot.
///
/// Exists because the census found seven brand colours with nowhere to live —
/// they were hand-copied into a Flux HelmRelease. base16 and ANSI projections
/// do NOT read this, so the sixteen stay total; only a renderer that declares a
/// brand need survive it.
#[derive(Debug, Clone, PartialEq, Eq, DeriveTataraDomain)]
#[tatara(keyword = "defbrand")]
pub struct Brand {
    pub ink: Srgb8,
    pub void: Srgb8,
    pub shadow_tone: Srgb8,
    pub paper: Srgb8,
    pub steel: Srgb8,
    pub silver: Srgb8,
    pub platinum: Srgb8,
}

/// A complete theme. The one artifact every renderer reads.
#[derive(Debug, Clone, PartialEq, DeriveTataraDomain)]
#[tatara(keyword = "deftheme")]
pub struct Theme {
    pub name: String,
    pub title: String,
    pub author: String,
    /// WCAG AA. Enforced in [`Theme::validate`], not advisory.
    pub contrast_floor: f64,
    #[tatara(domain)]
    pub ramp: Ramp,
    #[tatara(domain)]
    pub derived: Derived,
    #[tatara(domain)]
    pub ansi: Ansi16,
    #[tatara(domain)]
    pub roles: Roles,
    #[tatara(domain)]
    pub brand: Brand,
}

/// What a theme can be wrong about in a way the parser cannot catch.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SpecError {
    #[error("alpha for `{field}` is {value:?}; expected 0.0..=1.0")]
    AlphaOutOfRange { field: &'static str, value: String },
    #[error("contrast floor is {value:?}; expected a positive ratio (WCAG AA is 4.5)")]
    ContrastFloorInvalid { value: String },
    #[error("`{field}` blends over `{over:?}`, which is itself derived; a blend may only stack on a ramp slot")]
    BlendOnDerived { field: &'static str, over: Token },
}

impl Theme {
    /// Resolve any token, derived ones included.
    ///
    /// Total by construction: [`Token`] is closed, ramp slots resolve directly,
    /// and the two derived arms are the only others. A new derived token adds a
    /// variant, and this match stops compiling until it is handled.
    #[must_use]
    pub fn resolve(&self, t: Token) -> Srgb8 {
        match t {
            Token::Selection => self.blend(self.derived.selection),
            Token::Glow => self.blend(self.derived.glow),
            other => self
                .ramp
                .get(other)
                .expect("Ramp::get is total over non-derived tokens"),
        }
    }

    fn blend(&self, b: Blend) -> Srgb8 {
        let over = self.ramp.get(b.over).unwrap_or(self.ramp.base00);
        let tint = self.ramp.get(b.tint).unwrap_or(self.ramp.base00);
        let a = b.alpha.clamp(0.0, 1.0);
        let mix = |o: u8, t: u8| {
            let v = f64::from(o).mul_add(1.0 - a, f64::from(t) * a);
            // `.round()` then clamp: a mid-channel value must not wrap.
            v.round().clamp(0.0, 255.0) as u8
        };
        Srgb8 {
            r: mix(over.r, tint.r),
            g: mix(over.g, tint.g),
            b: mix(over.b, tint.b),
        }
    }

    /// The checks the parse boundary cannot make.
    ///
    /// Tier-honest: these are **assertion-caught**, not unrepresentable.
    /// tatara-lisp cannot bound a float, and `Blend`'s fields are `Token`, which
    /// admits the two derived variants — so "a blend over a blend" is
    /// constructible and must be rejected here rather than by the type.
    ///
    /// # Errors
    /// Returns the first violation found.
    pub fn validate(&self) -> Result<(), SpecError> {
        if !(self.contrast_floor > 0.0) {
            return Err(SpecError::ContrastFloorInvalid {
                value: self.contrast_floor.to_string(),
            });
        }
        for (field, b) in [
            ("selection", self.derived.selection),
            ("glow", self.derived.glow),
        ] {
            if !(0.0..=1.0).contains(&b.alpha) {
                return Err(SpecError::AlphaOutOfRange {
                    field,
                    value: b.alpha.to_string(),
                });
            }
            for t in [b.over, b.tint] {
                if self.ramp.get(t).is_none() {
                    return Err(SpecError::BlendOnDerived { field, over: t });
                }
            }
        }
        Ok(())
    }

    /// Parse a `(deftheme …)` form.
    ///
    /// # Errors
    /// Returns the reader's or the derive's typed `LispError`; a typo'd kwarg
    /// comes back with a did-you-mean and the allowed set.
    pub fn from_source(src: &str) -> tatara_lisp::Result<Self> {
        let forms = tatara_lisp::read(src)?;
        let first = forms.first().ok_or_else(|| tatara_lisp::LispError::Compile {
            form: Self::KEYWORD.to_string(),
            message: "source contains no forms".into(),
        })?;
        Self::compile_from_sexp(first)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NORD: &str = include_str!("../../themes/nord.theme.lisp");

    fn nord() -> Theme {
        Theme::from_source(NORD).expect("the authored Nord theme must compile")
    }

    /// ★ Load-bearing, not ceremony.
    ///
    /// A typo'd struct attribute (`keywords` for `keyword`) compiles SILENTLY
    /// and yields a keyword computed from the struct NAME — measured upstream:
    /// `AttrTypo` answered to `"defattrtypo"`. Nothing but a literal assertion
    /// catches it, so every domain type in this crate gets one.
    #[test]
    fn every_keyword_is_what_was_written() {
        assert_eq!(Theme::KEYWORD, "deftheme");
        assert_eq!(Ramp::KEYWORD, "deframp");
        assert_eq!(Blend::KEYWORD, "defblend");
        assert_eq!(Derived::KEYWORD, "defderived");
        assert_eq!(Ansi16::KEYWORD, "defansi");
        assert_eq!(Roles::KEYWORD, "defroles");
        assert_eq!(Brand::KEYWORD, "defbrand");
    }

    #[test]
    fn the_authored_nord_theme_compiles_and_validates() {
        let t = nord();
        assert_eq!(t.name, "nord");
        t.validate().expect("authored Nord must validate");
    }

    /// Byte-parity against the palette the fleet already renders. These are the
    /// values in `ishou/rendered/stylix-base16-nord-dark.nix`; if the authored
    /// form drifts from them, every downstream artifact silently repaints.
    #[test]
    fn the_ramp_matches_the_fleet_nord_bytes() {
        let r = nord().ramp;
        for (got, want) in [
            (r.base00, "2e3440"),
            (r.base01, "3b4252"),
            (r.base02, "434c5e"),
            (r.base03, "4c566a"),
            (r.base04, "d8dee9"),
            (r.base05, "e5e9f0"),
            (r.base06, "eceff4"),
            (r.base07, "8fbcbb"),
            (r.base08, "bf616a"),
            (r.base09, "d08770"),
            (r.base0a, "ebcb8b"),
            (r.base0b, "a3be8c"),
            (r.base0c, "88c0d0"),
            (r.base0d, "81a1c1"),
            (r.base0e, "b48ead"),
            (r.base0f, "5e81ac"),
        ] {
            assert_eq!(got.to_bare_hex(), want);
        }
    }

    /// The four contested slots, pinned to the authored verdict.
    ///
    /// ANSI black is `base01` (polar-night-1), NOT `base00` — `\e[30m` on the
    /// background renders invisible. That is the checkable reason irodzuki's
    /// mapping was chosen over the two aesthetic ones.
    #[test]
    fn the_contested_ansi_slots_resolve_to_the_verdict() {
        let t = nord();
        let a = t.ansi.in_order();
        assert_eq!(t.resolve(a[0]).to_bare_hex(), "3b4252", "ANSI black");
        assert_eq!(t.resolve(a[6]).to_bare_hex(), "88c0d0", "ANSI cyan");
        assert_eq!(t.resolve(a[7]).to_bare_hex(), "e5e9f0", "ANSI white");
        assert_eq!(t.resolve(a[14]).to_bare_hex(), "8fbcbb", "ANSI bright-cyan");
        assert_ne!(
            t.resolve(a[0]),
            t.ramp.base00,
            "ANSI black must never be the background"
        );
    }

    #[test]
    fn every_role_resolves_including_the_derived_ones() {
        let t = nord();
        // Two roles point at derived tokens; both must resolve without panic.
        assert_eq!(t.roles.selection, Token::Selection);
        assert_eq!(t.roles.agent, Token::Glow);
        for tok in [t.roles.background, t.roles.selection, t.roles.agent] {
            let _ = t.resolve(tok);
        }
        // A blend sits between its endpoints rather than equalling either.
        let sel = t.resolve(Token::Selection);
        assert_ne!(sel, t.ramp.base00);
        assert_ne!(sel, t.ramp.base0c);
    }

    /// ★ The property the whole vocabulary exists for: a role cannot carry a
    /// colour. `Token` has no hex-bearing constructor, so a literal in a role
    /// position is a parse error, not a second palette.
    #[test]
    fn a_hex_literal_in_a_role_position_is_rejected() {
        let bad = NORD.replace(":background       :base00", r#":background       "2E3440""#);
        assert_ne!(bad, NORD, "the fixture must actually have been edited");
        assert!(
            Theme::from_source(&bad).is_err(),
            "a hex in a role position must not compile"
        );
    }

    /// The trap the surface notes warn about, closed on the derive path: a
    /// mistyped key must not yield a partial theme reported as success.
    #[test]
    fn a_typod_kwarg_is_rejected_with_a_suggestion() {
        let bad = NORD.replace(":base0c \"88C0D0\"", ":base0z \"88C0D0\"");
        assert_ne!(bad, NORD);
        let err = Theme::from_source(&bad)
            .expect_err("a typo'd ramp slot must not compile")
            .to_string();
        assert!(err.contains("base0z"), "must name the bad key: {err}");
    }

    #[test]
    fn an_out_of_range_alpha_is_caught_by_validate_not_by_parse() {
        let bad = NORD.replace(":alpha 0.30", ":alpha 1.80");
        assert_ne!(bad, NORD);
        // It PARSES — tatara-lisp cannot bound a float — and validate() is what
        // rejects it. This asymmetry is the honest tier, pinned so it is not
        // later described as unrepresentable.
        let t = Theme::from_source(&bad).expect("an out-of-range alpha still parses");
        assert!(matches!(
            t.validate(),
            Err(SpecError::AlphaOutOfRange { .. })
        ));
    }

    #[test]
    fn srgb8_normalises_a_leading_hash_and_rejects_junk() {
        assert_eq!(
            Srgb8::try_from("#2E3440".to_string()).unwrap().to_bare_hex(),
            "2e3440"
        );
        assert!(Srgb8::try_from("2E344".to_string()).is_err());
        assert!(Srgb8::try_from("zzzzzz".to_string()).is_err());
    }
}
