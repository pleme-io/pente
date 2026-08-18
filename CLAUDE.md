# pente — the fleet theme vocabulary

`pending-keyword: deftheme (3 incumbents, pente unregistered)`

> **Status: M0, local. The GitHub repo does not exist yet.**
> Declared in `pangea-architectures` `org.yaml` and `repo-forge` `repos.lisp`
> (both pushed 2026-08-18), but all five `pleme-io-opensource` shards are
> `suspend: true` — the recorded shared-PAT park, whose exit condition
> (pangea-operator `5391325`) is not met. Declared ≠ created; do not cite this
> repo as existing on GitHub until `gh repo view pleme-io/pente` says so.

A theme is authored **once**, as a `(deftheme …)` tatara-lisp form over a typed
Rust border, and rendered to every consumer. The ramp in
`themes/nord.theme.lisp` is the only place in the fleet a colour literal may
appear.

## Why

A census on 2026-08-18 measured **118 palette copies** across the fleet in
**eight distinct encodings** — with an unknown denominator (~62 repos carry ≥5
colour literals; the census reached ~26, and 8 of the misses are root flake
inputs). The encodings matter more than the count, because a `#RRGGBB` grep sees
only three of them: bare hex `"2E3440"`, `Rgb::new(0x2e,…)`, decimal
`Color::new(136,192,208)`, float `vec3(0.53,…)`, `[f32;4]`, and decimal ANSI
escapes are all invisible to it. Every gate proposed before that measurement
would have read green while missing most of the corpus.

The source of truth had already drifted from itself: **four** Nord ANSI-16
tables exist, four slots disagree, and slot 7 is a three-way split.

## The two rules

1. **A colour is named, never spelled** — outside the ramp. The role and ANSI
   forms take a `Token` keyword, and `Token` has no hex-bearing constructor, so
   `:background "2E3440"` is a parse error listing the tokens that exist.
2. **The ANSI table is authored, not derived.** It is a convention Nord
   publishes, not a function of the ramp — which is precisely why four
   independent derivations disagree. See `themes/nord.theme.lisp` for the
   verdict and its reasoning (ANSI black must not be the background, or
   `\e[30m` renders invisible).

## Substrate facts this border depends on

Proven by a compile probe now living upstream as tatara-lisp's
`tests/phase_f_constructs.rs` (tatara-lisp@`6f2056a`). Before it,
`#[tatara(domain)]`, `#[tatara(keyword_enum)]` and `#[derive(KeywordSexp)]` had
**zero use sites fleet-wide** against 202 files using `DeriveTataraDomain` —
implemented, never compiled.

- A typo'd **kwarg** is rejected, with a did-you-mean and the allowed set. The
  surface notes' "empty `Vec` reported as success" warning describes the
  *manual extraction* path, not the derive path.
- A typo'd **struct attribute** is **silent**: `#[tatara(keywords = …)]` yields
  a keyword computed from the struct NAME. This is why `every_keyword_is_what_was_written`
  asserts each `KEYWORD` literally — that test is load-bearing, not ceremony.
- Two **opposite** naming rules: `KeywordSexp` lowercases the ident with no
  separator (`Base0A` → `:base0a`); field names take `snake_to_kebab`
  (`bright_black` → `:bright-black`).

## Divergence from the spec, recorded

`docs/pente-vocabulary-spec.md` (in `pleme-io/nix`) authors both `:ansi` and
`:roles` as `defbinding`. That cannot be built: they carry different key sets,
so one struct cannot hold both, and two structs sharing a keyword collide at
registration. They are `defansi` and `defroles` here.

## Tier honesty

- `Token` having no hex constructor → **truly-unrepresentable**.
- A typo'd kwarg → **parse-time-rejected**.
- An out-of-range `alpha`, and a blend stacked on a blend → **only-mitigated
  (C1)**; tatara-lisp cannot bound a float and `Blend`'s fields are `Token`,
  which admits the derived variants. `Theme::validate` catches both, and
  `an_out_of_range_alpha_is_caught_by_validate_not_by_parse` pins the asymmetry
  so it is not later described as unrepresentability.
