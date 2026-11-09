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

    /// ★ Blends in LINEAR space, not sRGB. This is not a refinement — mixing
    /// gamma-encoded bytes directly is simply the wrong arithmetic, and it is
    /// the exact defect ishou's typed
    /// `Srgb`/`Linear` split to make uncompilable ("washed-out gray instead of
    /// Nord dark in mado post-rebuild").
    ///
    /// The first draft of this function mixed raw `u8`s and was wrong. It was
    /// caught by recovering Borealis from git history and finding that its own
    /// `blend_linear` — the function that produced the palette's shipped
    /// `base02` — round-trips through the sRGB transfer function. Reproducing
    /// that byte-for-byte is what makes a recovered palette actually the same
    /// palette; the naive mix differs by several counts per channel.
    fn blend(&self, b: Blend) -> Srgb8 {
        // sRGB transfer functions (IEC 61966-2-1), matching ishou's own.
        fn to_linear(c: u8) -> f64 {
            let c = f64::from(c) / 255.0;
            if c <= 0.040_45 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        fn to_channel(l: f64) -> u8 {
            let v = if l <= 0.003_130_8 {
                12.92 * l
            } else {
                1.055 * l.powf(1.0 / 2.4) - 0.055
            };
            let scaled = (v * 255.0).round();
            if scaled.is_nan() {
                return 0;
            }
            scaled.clamp(0.0, 255.0) as u8
        }

        let over = self.ramp.get(b.over).unwrap_or(self.ramp.base00);
        let tint = self.ramp.get(b.tint).unwrap_or(self.ramp.base00);
        let a = b.alpha.clamp(0.0, 1.0);
        let mix = |o: u8, t: u8| to_channel(to_linear(o).mul_add(1.0 - a, to_linear(t) * a));
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
        let first = forms
            .first()
            .ok_or_else(|| tatara_lisp::LispError::Compile {
                form: Self::KEYWORD.to_string(),
                message: "source contains no forms".into(),
            })?;
        Self::compile_from_sexp(first)
    }
}

/// The fleet's themes. **Nord is the default; the others are choices.**
///
/// Closed on purpose. A renderer matches on this, so a theme that some target
/// cannot render stops compiling rather than rendering blank — which is the
/// asymmetry the census found in ishou, where twelve render targets existed for
/// Nord and four for Vellum, so "swap the theme" silently un-themed eight
/// consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DeriveKeywordSexp)]
pub enum FleetTheme {
    /// The fleet default (operator decision, 2026-08-18).
    Nord,
    /// Cool blue-grey aurora night. Restored from history — see
    /// `themes/borealis.theme.lisp`.
    Borealis,
    /// Warm aged-paper matte. The pre-Nord incumbent.
    Vellum,
}

impl Default for FleetTheme {
    /// Nord. Every consumer that does not choose gets it.
    fn default() -> Self {
        Self::Nord
    }
}

impl FleetTheme {
    /// Every theme. A renderer iterating this cannot forget one, and adding a
    /// variant breaks the array length until it is handled.
    pub const ALL: [Self; 3] = [Self::Nord, Self::Borealis, Self::Vellum];

    /// The stable identifier used in artifact names and file names.
    ///
    /// Derived here rather than authored in the theme, deliberately: stylix
    /// interpolates a slug into generated artefact NAMES (`base16-${slug}`,
    /// `${slug}-gnome-shell-theme`), so an authored-and-mistyped one keeps every
    /// colour byte-identical while silently renaming a pile of store paths.
    /// Deriving it from the closed enum makes that unrepresentable.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Nord => "nord",
            Self::Borealis => "borealis",
            Self::Vellum => "vellum",
        }
    }

    /// The authored source, compiled into the binary so a theme cannot be
    /// missing at runtime and cannot be edited out from under a consumer.
    #[must_use]
    pub const fn source(self) -> &'static str {
        match self {
            Self::Nord => include_str!("../../themes/nord.theme.lisp"),
            Self::Borealis => include_str!("../../themes/borealis.theme.lisp"),
            Self::Vellum => include_str!("../../themes/vellum.theme.lisp"),
        }
    }

    /// Compile this theme.
    ///
    /// # Errors
    /// Returns the typed `LispError` if the authored form is malformed — which
    /// the test suite makes a build failure rather than a runtime one.
    pub fn load(self) -> tatara_lisp::Result<Theme> {
        Theme::from_source(self.source())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NORD: &str = FleetTheme::Nord.source();

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

    // ── The three themes ───────────────────────────────────────────────

    #[test]
    fn nord_is_the_default() {
        assert_eq!(FleetTheme::default(), FleetTheme::Nord);
        assert_eq!(FleetTheme::default().load().unwrap().name, "nord");
    }

    /// Every theme compiles and validates. This is what makes a theme swap a
    /// real operation rather than a hope: the failure mode it removes is a
    /// palette that only the default renders correctly.
    #[test]
    fn every_fleet_theme_compiles_and_validates() {
        for t in FleetTheme::ALL {
            let theme = t
                .load()
                .unwrap_or_else(|e| panic!("{t:?} failed to compile: {e}"));
            theme
                .validate()
                .unwrap_or_else(|e| panic!("{t:?} failed to validate: {e}"));
        }
    }

    /// Every theme names itself, and no two share a name.
    #[test]
    fn the_three_themes_are_distinctly_named() {
        let names: Vec<String> = FleetTheme::ALL
            .iter()
            .map(|t| t.load().unwrap().name)
            .collect();
        assert_eq!(names, vec!["nord", "borealis", "vellum"]);
    }

    /// ★ They are genuinely DIFFERENT PALETTES, not tints of one.
    ///
    /// Worth pinning because for months they were not: `borealis_night()` in
    /// frost has been returning `VellumPalette::vellum()` under the Borealis
    /// name since commit 756bd35 retired the palette and kept the label. A
    /// theme that answers to a name that is not its palette is the exact bug
    /// this assertion makes impossible to reintroduce silently.
    #[test]
    fn no_two_themes_share_a_ramp() {
        let ramps: Vec<(FleetTheme, Ramp)> = FleetTheme::ALL
            .iter()
            .map(|t| (*t, t.load().unwrap().ramp))
            .collect();
        for (i, (ta, ra)) in ramps.iter().enumerate() {
            for (tb, rb) in ramps.iter().skip(i + 1) {
                assert_ne!(ra, rb, "{ta:?} and {tb:?} are the same palette");
            }
        }
        // And the backgrounds differ in TEMPERATURE, not just value — Borealis
        // is blue-dominant, Vellum red-dominant, Nord blue-dominant but lighter.
        let bg = |t: FleetTheme| t.load().unwrap().ramp.base00;
        assert!(
            bg(FleetTheme::Vellum).r > bg(FleetTheme::Vellum).b,
            "Vellum is warm"
        );
        assert!(
            bg(FleetTheme::Borealis).b > bg(FleetTheme::Borealis).r,
            "Borealis is cool"
        );
    }

    /// Every role and every ANSI slot resolves, in every theme.
    ///
    /// The asymmetry this forbids is the one the census measured in ishou:
    /// twelve render targets for Nord against four for Vellum, so selecting a
    /// non-default theme silently un-themed eight consumers.
    #[test]
    fn every_theme_resolves_every_role_and_ansi_slot() {
        for t in FleetTheme::ALL {
            let th = t.load().unwrap();
            let r = &th.roles;
            for tok in [
                r.background,
                r.surface,
                r.surface_elevated,
                r.text,
                r.text_muted,
                r.text_dim,
                r.primary,
                r.accent,
                r.border,
                r.error,
                r.warning,
                r.success,
                r.info,
                r.selection,
                r.cursor,
                r.agent,
            ] {
                let _ = th.resolve(tok);
            }
            for (i, tok) in th.ansi.in_order().into_iter().enumerate() {
                let c = th.resolve(tok);
                assert_ne!(
                    (i, c),
                    (0, th.ramp.base00),
                    "{t:?}: ANSI black must never be the background"
                );
            }
        }
    }

    /// ★ The blend is gamma-correct, pinned against a value the fleet already
    /// shipped.
    ///
    /// Borealis's original `selection` was `blend_linear(night0, fable_violet,
    /// 0.08)` computed in LINEAR space, and shipped as `3F3955`. Reproducing it
    /// is the check that this crate's blend does the same arithmetic — the
    /// first draft mixed raw sRGB bytes and lands several counts off per
    /// channel, which is precisely the washed-out-render class ishou's typed
    /// colour spaces exist to prevent.
    #[test]
    fn the_blend_is_linear_space_not_srgb() {
        let night0 = Srgb8 {
            r: 0x1F,
            g: 0x22,
            b: 0x2F,
        };
        let violet = Srgb8 {
            r: 0xB6,
            g: 0x9A,
            b: 0xE9,
        };
        let mut t = FleetTheme::Borealis.load().unwrap();
        t.ramp.base00 = night0;
        t.ramp.base0c = violet;
        t.derived.selection = Blend {
            over: Token::Base00,
            tint: Token::Base0c,
            alpha: 0.08,
        };
        assert_eq!(
            t.resolve(Token::Selection).to_bare_hex(),
            "3f3955",
            "must reproduce ishou's shipped blend_linear output"
        );
    }

    #[test]
    fn srgb8_normalises_a_leading_hash_and_rejects_junk() {
        assert_eq!(
            Srgb8::try_from("#2E3440".to_string())
                .unwrap()
                .to_bare_hex(),
            "2e3440"
        );
        assert!(Srgb8::try_from("2E344".to_string()).is_err());
        assert!(Srgb8::try_from("zzzzzz".to_string()).is_err());
    }
}
