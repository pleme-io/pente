//! `pente-forge` — regenerate every consumer artifact from the authored themes.
//!
//! Two verbs, deliberately:
//!
//! - `build <dir>` writes the whole matrix (every theme × every target).
//! - `check <dir>` regenerates in memory and DIFFS against what is on disk,
//!   exiting non-zero on any drift.
//!
//! `check` is the one that matters. Generated files carry "DO NOT EDIT"
//! headers, and across the fleet that has been advisory: three
//! `module/themes/nord/colors.nix` files carry exactly that header and nothing
//! has ever checked them. A header is a wish; a differential is a gate.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use pente_forge::sdk::{corpus, Lang};
use pente_forge::Target;
use pente_spec::FleetTheme;

fn usage() -> ExitCode {
    eprintln!(
        "pente-forge — regenerate consumer artifacts from the authored themes\n\
         \n\
         USAGE:\n    \
             pente-forge build <out-dir>    write every theme × every target\n    \
             pente-forge check <out-dir>    diff against disk; non-zero on drift\n    \
             pente-forge list               print the matrix without writing\n"
    );
    ExitCode::from(2)
}

/// Every artifact, rendered. The single place the matrix is enumerated, so
/// `build` and `check` cannot disagree about what exists.
fn matrix() -> Result<Vec<(PathBuf, String)>, String> {
    let mut out = Vec::new();
    for theme in FleetTheme::ALL {
        let t = theme
            .load()
            .map_err(|e| format!("theme {theme:?} failed to compile: {e}"))?;
        t.validate()
            .map_err(|e| format!("theme {theme:?} failed to validate: {e}"))?;
        for target in Target::ALL {
            out.push((
                PathBuf::from(target.file_name(theme)),
                target.render(theme, &t),
            ));
        }
        // The SDK modules live under sdk/<lang>/, one per theme.
        for lang in Lang::ALL {
            out.push((
                PathBuf::from("sdk").join(lang.name()).join(lang.module_name(theme)),
                lang.render_module(theme, &t),
            ));
        }
    }

    // Per-LANGUAGE, not per-theme: each conformance test iterates the corpus,
    // so a new theme is covered without a new test file — the property that
    // stops per-language coverage decaying as themes are added.
    for lang in Lang::ALL {
        let dir = PathBuf::from("sdk").join(lang.name());
        out.push((dir.join(lang.test_name()), lang.render_test()));
        for (name, body) in lang.scaffold() {
            out.push((dir.join(name), body));
        }
        // The corpus is emitted BESIDE each language's test rather than once at
        // the root, because every one of these test runners resolves fixtures
        // relative to its own directory. One shared copy would work for exactly
        // whichever runner was configured first.
        out.push((dir.join("corpus.json"), corpus()));
    }
    Ok(out)
}

fn build(dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for (name, body) in matrix()? {
        let path = dir.join(&name);
        // Artifact names carry directories (sdk/<lang>/…), so each parent is
        // created rather than assumed. Missed on the first pass: the write
        // failed silently-ish after five files, which is exactly the shape of
        // bug a matrix test that only checks the flat targets would not catch.
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        std::fs::write(&path, body).map_err(|e| format!("{}: {e}", path.display()))?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

fn check(dir: &Path) -> Result<usize, String> {
    let mut drift = 0usize;
    for (name, want) in matrix()? {
        let path = dir.join(&name);
        match std::fs::read_to_string(&path) {
            Err(_) => {
                eprintln!("MISSING  {}", path.display());
                drift += 1;
            }
            Ok(got) if got != want => {
                // Name the first differing line: a whole-file diff of a colour
                // table is unreadable, and the useful fact is almost always one
                // slot.
                let at = got
                    .lines()
                    .zip(want.lines())
                    .position(|(a, b)| a != b)
                    .map_or_else(|| "length".to_string(), |i| format!("line {}", i + 1));
                eprintln!("DRIFT    {} (first difference at {at})", path.display());
                drift += 1;
            }
            Ok(_) => {}
        }
    }
    Ok(drift)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let run = || -> Result<ExitCode, String> {
        match args.first().map(String::as_str) {
            Some("list") => {
                for (name, body) in matrix()? {
                    println!("{:<28} {:>6} bytes", name.display(), body.len());
                }
                Ok(ExitCode::SUCCESS)
            }
            Some("build") => {
                let dir = args.get(1).ok_or("build needs an <out-dir>")?;
                build(Path::new(dir))?;
                Ok(ExitCode::SUCCESS)
            }
            Some("check") => {
                let dir = args.get(1).ok_or("check needs an <out-dir>")?;
                let drift = check(Path::new(dir))?;
                if drift == 0 {
                    println!("pente-forge check: no drift");
                    Ok(ExitCode::SUCCESS)
                } else {
                    eprintln!(
                        "pente-forge check: {drift} artifact(s) drifted. Regenerate:\n  \
                         pente-forge build <out-dir>"
                    );
                    Ok(ExitCode::FAILURE)
                }
            }
            _ => Ok(usage()),
        }
    };
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("pente-forge: {e}");
            ExitCode::FAILURE
        }
    }
}
