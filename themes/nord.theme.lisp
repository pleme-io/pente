;; ─────────────────────────────────────────────────────────────────────
;; THE SOURCE ARTIFACT. Nord — the standardized fleet theme.
;;
;; Every palette, ANSI table and blend in the fleet is generated from this
;; file and byte-pinned to it. A census on 2026-08-18 found 118 palette
;; copies in eight distinct encodings, with an unknown denominator; this is
;; the source that replaces them.
;;
;; The ramp below is the ONLY place in the fleet a colour literal may
;; appear. Everywhere else a colour is named, never spelled — the role and
;; ANSI forms take a token keyword and have no hex-bearing constructor, so a
;; stray literal is a parse error rather than copy 119.
;; ─────────────────────────────────────────────────────────────────────

(deftheme
  :name           "nord"
  :title          "Nord (pleme-io / pente)"
  :author         "Arctic Ice Studio; roles mapped by pleme-io pente"
  :contrast-floor 4.5              ; WCAG AA — enforced in validate(), not advisory

  ;; NOTE: `variant` and `slug` are NOT authored — both are DERIVED. stylix
  ;; interpolates them into artefact NAMES (`base16-${slug}`,
  ;; `${slug}-gnome-shell-theme`), so a defaulted "untitled"/"unspecified"
  ;; keeps every colour byte-identical while renaming a pile of store paths.

  ;; ── THE RAMP ───────────────────────────────────────────────────────
  ;; base16-indexed, not Nord-named: `polar_night_0` is Nord's own
  ;; vocabulary, and a substrate that names it cannot host a second palette.
  ;; Nord's proper nouns live in the comments and are re-exported as a
  ;; generated alias module, never authored here.
  ;;
  ;; All sixteen mandatory. Bare hex, no '#' — stylix passes the value
  ;; through to GTK, so a '#' is a silent colour change rather than an error.
  ;; Transcribed byte-for-byte from ishou/rendered/stylix-base16-nord-dark.nix.
  :ramp (deframp
          :base00 "2E3440"   ; polar-night-0   background
          :base01 "3B4252"   ; polar-night-1   surface, ANSI black
          :base02 "434C5E"   ; polar-night-2   selection ground
          :base03 "4C566A"   ; polar-night-3   comments, ANSI bright-black
          :base04 "D8DEE9"   ; snow-storm-0    muted foreground
          :base05 "E5E9F0"   ; snow-storm-1    ANSI white
          :base06 "ECEFF4"   ; snow-storm-2    foreground, ANSI bright-white
          :base07 "8FBCBB"   ; frost-0         (see NOTE below)
          :base08 "BF616A"   ; aurora-red      error
          :base09 "D08770"   ; aurora-orange   constants
          :base0a "EBCB8B"   ; aurora-yellow   warning
          :base0b "A3BE8C"   ; aurora-green    success
          :base0c "88C0D0"   ; frost-1         ANSI cyan
          :base0d "81A1C1"   ; frost-2         ANSI bright-blue
          :base0e "B48EAD"   ; aurora-purple   ANSI magenta
          :base0f "5E81AC")  ; frost-3         ANSI blue

  ;; NOTE base07: canonical Nord puts a frost CYAN where base16's convention
  ;; says "lightest background / inverse". That is upstream Nord's choice and
  ;; byte-parity with the fleet's existing render forces us to reproduce it.
  ;; Do not "fix" it — every downstream artifact is pinned to these bytes.

  ;; ── DERIVED TOKENS ─────────────────────────────────────────────────
  ;; Recipes, never literals. tatara-lisp has no arithmetic at the compile
  ;; layer, so a blend is DATA and Theme::resolve is the one evaluator —
  ;; which is also what stops "a slightly lighter background" becoming the
  ;; next hand-typed copy.
  ;;
  ;; A blend may only stack on a RAMP slot; blending over a blend is rejected
  ;; by validate(), so resolution is one pass with no cycle to detect.
  :derived (defderived
             :selection (defblend :over :base00 :tint :base0c :alpha 0.30)
             :glow      (defblend :over :base00 :tint :base0e :alpha 0.18))

  ;; ── THE ANSI-16 TABLE ──────────────────────────────────────────────
  ;; AUTHORED ONCE, as index -> ramp token. NOT derived, deliberately: the
  ;; mapping is a convention Nord publishes, not a function of the ramp —
  ;; which is exactly why FOUR independent derivations of it exist in the
  ;; fleet today and four slots disagree, slot 7 three ways:
  ;;
  ;;   idx           ghostty.rs   fleet_theme.rs   irodzuki
  ;;   0  black      2E3440       2E3440           3B4252
  ;;   6  cyan       8FBCBB       88C0D0           88C0D0
  ;;   7  white      D8DEE9       ECEFF4           E5E9F0
  ;;   14 br-cyan    88C0D0       8FBCBB           8FBCBB
  ;;
  ;; VERDICT: irodzuki's values, on a checkable reason rather than taste —
  ;; it is the only one of the three citing an upstream (nordtheme.com's
  ;; terminal mapping), and ANSI black must be polar-night-1 rather than the
  ;; BACKGROUND, or `\e[30m` renders invisible. irodzuki's own comment
  ;; claiming it "renders byte-identically to ghostty" is false on all four.
  ;;
  ;; Every terminal renderer projects THIS array; none may derive its own.
  :ansi (defansi
          :black          :base01   :red            :base08
          :green          :base0b   :yellow         :base0a
          :blue           :base0f   :magenta        :base0e
          :cyan           :base0c   :white          :base05
          :bright-black   :base03   :bright-red     :base08
          :bright-green   :base0b   :bright-yellow  :base0a
          :bright-blue    :base0d   :bright-magenta :base0e
          :bright-cyan    :base07   :bright-white   :base06)

  ;; ── SEMANTIC ROLES ─────────────────────────────────────────────────
  ;; What a consumer asks for by MEANING. A role's value is a TOKEN keyword,
  ;; never a hex: the field type has no hex-bearing constructor, so a literal
  ;; here is a parse error listing the tokens that exist.
  :roles (defroles
           :background       :base00
           :surface          :base01
           :surface-elevated :base02
           :text             :base06
           :text-muted       :base04
           :text-dim         :base03
           :primary          :base0c
           :accent           :base0f
           :border           :base03
           :error            :base08
           :warning          :base0a
           :success          :base0b
           :info             :base0d
           :selection        :selection
           :cursor           :base04
           :agent            :glow)

  ;; ── THE BRAND EXTENSION ────────────────────────────────────────────
  ;; Seven colours that are NOT Nord and have no base16 slot. They exist
  ;; because the census found them with nowhere to live — hand-copied into a
  ;; Flux HelmRelease, reachable by no Nix or Cargo path.
  ;;
  ;; base16 and ANSI projections do NOT read this form, so the sixteen stay
  ;; total; only a renderer that declares a brand need survive it.
  :brand (defbrand
           :ink         "0A0A0A"
           :void        "0A0A0B"
           :shadow-tone "141822"
           :paper       "F5F5F0"
           :steel       "6B7280"
           :silver      "9AA3B2"
           :platinum    "C8CDD8"))
