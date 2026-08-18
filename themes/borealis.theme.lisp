;; ─────────────────────────────────────────────────────────────────────
;; BOREALIS — a selectable fleet theme. Cool blue-grey aurora night.
;;
;; ★ RESTORED, not invented. Borealis was a real palette
;; (ishou-tokens `BorealisPalette::night`) until commit 756bd35, "theme:
;; replace Borealis with Vellum". That commit deleted
;; crates/ishou-tokens/src/borealis.rs — and left the NAME behind pointing at
;; a different palette:
;;
;;     frost/crates/frost-lisp/src/theme.rs:120
;;       pub fn borealis_night() -> ThemeSpec {
;;           // Borealis was retired in favour of Vellum …
;;           use ishou_tokens::VellumPalette;
;;           let p = VellumPalette::vellum();
;;
;; So the operator's login shell has been rendering VELLUM under the name
;; "borealis-night" ever since — a theme that answers to a name that is not
;; its palette. Making Borealis a distinct theme again is what removes that.
;;
;; Every value below is recovered from `git show 756bd35^` and transcribed
;; byte-for-byte. Nothing here is a guess.
;; ─────────────────────────────────────────────────────────────────────

(deftheme
  :name           "borealis"
  :title          "Borealis Night (pleme-io / pente)"
  :author         "pleme-io ishou — gamut-fitted from OKLCH targets"
  :contrast-floor 4.5

  ;; base16 slots via the recovered `BorealisPalette::base16()` mapping:
  ;;   base00 night0     base04 snow0    base08 aurora_red     base0c ice_cyan
  ;;   base01 night1     base05 snow1    base09 ember          base0d ice_steel
  ;;   base02 selection  base06 snow2    base0a first_light    base0e solar_magenta
  ;;   base03 shadow1    base07 snow3    base0b aurora_green   base0f dusk_bronze
  :ramp (deframp
          :base00 "1F222F"   ; night0          background — 1.47x lighter than Vellum, and BLUE where Vellum is warm
          :base01 "2B2E3D"   ; night1          surface
          :base02 "3F3955"   ; selection       ★ see NOTE
          :base03 "8C92A4"   ; shadow1         comments
          :base04 "B5BCC9"   ; snow0           muted foreground
          :base05 "D4D9E3"   ; snow1           foreground
          :base06 "E4E8EF"   ; snow2           bright foreground
          :base07 "F5F7FA"   ; snow3           lightest
          :base08 "D86E67"   ; aurora_red      error
          :base09 "E89772"   ; ember           constants
          :base0a "EDC980"   ; first_light     warning
          :base0b "67D191"   ; aurora_green    success
          :base0c "73C6D9"   ; ice_cyan        primary
          :base0d "6A90C0"   ; ice_steel       info
          :base0e "C673A3"   ; solar_magenta   keywords
          :base0f "B38363")  ; dusk_bronze     deprecated

  ;; NOTE base02: in the original, `selection` was not an authored constant —
  ;; it was `blend_linear(night0, fable_violet, 0.08)`, computed in LINEAR
  ;; space. The value above is that function's output, reproduced exactly.
  ;;
  ;; It is transcribed rather than expressed as a `defblend` for one reason:
  ;; `fable_violet` (#B69AE9) is NOT a base16 slot in this palette, and a
  ;; blend may only stack on a ramp slot. Recovering it as a literal keeps the
  ;; palette byte-faithful; expressing it as a recipe would require widening
  ;; the ramp to carry a token nothing else uses. Recorded so the next reader
  ;; knows this one value has a derivation, and what it was.

  ;; Derived tokens use THIS palette's own accents — not Nord's. That is the
  ;; point of a distinct theme: the recipes are shared, the inputs are not.
  :derived (defderived
             :selection (defblend :over :base00 :tint :base0c :alpha 0.30)
             :glow      (defblend :over :base00 :tint :base0e :alpha 0.18))

  ;; Same ANSI convention as Nord — black is surface, never background, or
  ;; `\e[30m` renders invisible.
  :ansi (defansi
          :black          :base01   :red            :base08
          :green          :base0b   :yellow         :base0a
          :blue           :base0f   :magenta        :base0e
          :cyan           :base0c   :white          :base05
          :bright-black   :base03   :bright-red     :base08
          :bright-green   :base0b   :bright-yellow  :base0a
          :bright-blue    :base0d   :bright-magenta :base0e
          :bright-cyan    :base07   :bright-white   :base06)

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

  ;; Brand colours are fleet identity, not palette — identical across themes
  ;; by design. A theme changes how the fleet LOOKS, not who it is.
  :brand (defbrand
           :ink         "0A0A0A"
           :void        "0A0A0B"
           :shadow-tone "141822"
           :paper       "F5F5F0"
           :steel       "6B7280"
           :silver      "9AA3B2"
           :platinum    "C8CDD8"))
