//! The committed `rendered/` tree vs. its generator.
//!
//! Those files carry "DO NOT EDIT" headers, and across this fleet that has been
//! advisory: three separate `module/themes/nord/colors.nix` files carry exactly
//! that header and nothing has ever checked them. A header is a wish; this is
//! the gate.
//!
//! Two distinct properties are proven here, and they are easy to conflate:
//!
//! 1. **Freshness** — the bytes on disk are what the generator emits today.
//! 2. **Compilability** — the emitted Rust is valid Rust *in the way consumers
//!    actually use it*. `include!` is that way, which is why the generator emits
//!    an item-level `#[allow]` rather than an inner `#![allow]`; an inner
//!    attribute is only legal at the top of a file or module, so emitting one
//!    would produce artifacts that look fine and break every consumer.
//!
//! A freshness test alone would have passed with uncompilable Rust.

use pente_forge::Target;
use pente_spec::FleetTheme;

/// ★ Compilability. Each generated constants file is `include!`d into its own
/// module — if any emitted a syntax error, this test target would not build.
mod generated {
    pub mod nord {
        include!("../../rendered/nord.rs");
    }
    pub mod borealis {
        include!("../../rendered/borealis.rs");
    }
    pub mod vellum {
        include!("../../rendered/vellum.rs");
    }
}

fn rendered_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("rendered")
}

/// Freshness, over the whole matrix.
#[test]
fn every_committed_artifact_matches_its_generator() {
    let dir = rendered_dir();
    let mut checked = 0usize;
    for theme in FleetTheme::ALL {
        let t = theme.load().expect("theme compiles");
        for target in Target::ALL {
            let name = target.file_name(theme);
            let path = dir.join(&name);
            let on_disk = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{name} is missing ({e}). Run: pente-forge build rendered"));
            assert_eq!(
                on_disk,
                target.render(theme, &t),
                "{name} is STALE. Regenerate with: pente-forge build rendered"
            );
            checked += 1;
        }
    }
    // The denominator, inside the compared value: a discovery bug that found
    // nothing would otherwise pass this test silently.
    assert_eq!(
        checked,
        FleetTheme::ALL.len() * Target::ALL.len(),
        "the matrix shrank"
    );
}

/// The generated constants carry the same values the spec resolves — the two
/// halves cannot drift.
#[test]
fn generated_constants_agree_with_the_spec() {
    let cases: [(FleetTheme, [u32; 16], [u32; 16]); 3] = [
        (FleetTheme::Nord, generated::nord::BASE16, generated::nord::ANSI16),
        (
            FleetTheme::Borealis,
            generated::borealis::BASE16,
            generated::borealis::ANSI16,
        ),
        (
            FleetTheme::Vellum,
            generated::vellum::BASE16,
            generated::vellum::ANSI16,
        ),
    ];
    for (theme, base16, ansi16) in cases {
        let t = theme.load().unwrap();
        let ramp = [
            t.ramp.base00, t.ramp.base01, t.ramp.base02, t.ramp.base03,
            t.ramp.base04, t.ramp.base05, t.ramp.base06, t.ramp.base07,
            t.ramp.base08, t.ramp.base09, t.ramp.base0a, t.ramp.base0b,
            t.ramp.base0c, t.ramp.base0d, t.ramp.base0e, t.ramp.base0f,
        ];
        for (i, c) in ramp.into_iter().enumerate() {
            let packed = (u32::from(c.r) << 16) | (u32::from(c.g) << 8) | u32::from(c.b);
            assert_eq!(base16[i], packed, "{theme:?} base16[{i}]");
        }
        for (i, tok) in t.ansi.in_order().into_iter().enumerate() {
            let c = t.resolve(tok);
            let packed = (u32::from(c.r) << 16) | (u32::from(c.g) << 8) | u32::from(c.b);
            assert_eq!(ansi16[i], packed, "{theme:?} ansi16[{i}]");
        }
    }
}

/// ★ The four contested ANSI slots, asserted on the GENERATED artifact.
///
/// The spec pins them at the source; this pins them at the thing consumers
/// actually read. Four independent derivations of this table disagreed before
/// it had one source, and slot 7 was a three-way split.
#[test]
fn the_generated_nord_ansi_carries_the_verdict() {
    let a = generated::nord::ANSI16;
    assert_eq!(a[0], 0x3B4252, "ANSI black is polar-night-1, not the background");
    assert_eq!(a[6], 0x88C0D0, "ANSI cyan");
    assert_eq!(a[7], 0xE5E9F0, "ANSI white");
    assert_eq!(a[14], 0x8FBCBB, "ANSI bright-cyan");
    assert_ne!(a[0], generated::nord::BASE16[0], "black must never equal the background");
}

/// No artifact may carry a `#RRGGBB` in a target whose wire format is bare hex.
#[test]
fn the_nix_artifacts_emit_bare_hex() {
    for theme in FleetTheme::ALL {
        let body = std::fs::read_to_string(rendered_dir().join(Target::NixBase16.file_name(theme)))
            .expect("artifact exists");
        let code: String = body
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !code.contains("\"#"),
            "{theme:?}: a leading '#' reaches GTK and changes the colour silently"
        );
    }
}
