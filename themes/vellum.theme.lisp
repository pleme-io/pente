;; ─────────────────────────────────────────────────────────────────────
;; VELLUM — a selectable fleet theme. Warm aged-paper, matte Nord.
;;
;; Not the default. Nord is (2026-08-18, operator decision), and Vellum is
;; one of the two alternatives alongside Borealis.
;;
;; Vellum is the fleet's INCUMBENT in one specific sense worth recording: it
;; is what stylix actually rendered on ggg and the Macs until the Nord
;; standardization, so it is the palette a reader will find in older
;; screenshots and in `ishou/rendered/stylix-base16-vellum.nix`. Values below
;; are transcribed byte-for-byte from that render.
;;
;; It is also the palette the name "borealis-night" has been pointing at in
;; frost since commit 756bd35 — see themes/borealis.theme.lisp for why that
;; matters and how it is undone.
;; ─────────────────────────────────────────────────────────────────────

(deftheme
  :name           "vellum"
  :title          "Vellum (pleme-io / pente)"
  :author         "pleme-io ishou — BORN palette, warm-neutral matte"
  :contrast-floor 4.5

  ;; Warm where Borealis is cool: base00 is 16140E (R > B) against Borealis's
  ;; 1F222F (B > R). The three fleet themes are genuinely distinct palettes,
  ;; not tints of one — which is what makes "swap the theme" a real test of
  ;; the vocabulary rather than a cosmetic one.
  :ramp (deframp
          :base00 "16140E"   ; background — warm, deepest of the three
          :base01 "1F1C15"   ; surface
          :base02 "3A343E"   ; selection ground
          :base03 "90897B"   ; comments
          :base04 "ADA593"   ; muted foreground
          :base05 "E2DBC8"   ; foreground
          :base06 "EDE6D6"   ; bright foreground
          :base07 "F4EFE2"   ; lightest
          :base08 "C9837B"   ; error
          :base09 "CB9070"   ; constants
          :base0a "D7C489"   ; warning
          :base0b "A9BB8C"   ; success
          :base0c "94BBB8"   ; primary
          :base0d "99AABE"   ; info
          :base0e "B8A1B9"   ; keywords
          :base0f "B3886C")  ; deprecated

  :derived (defderived
             :selection (defblend :over :base00 :tint :base0c :alpha 0.30)
             :glow      (defblend :over :base00 :tint :base0e :alpha 0.18))

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

  :brand (defbrand
           :ink         "0A0A0A"
           :void        "0A0A0B"
           :shadow-tone "141822"
           :paper       "F5F5F0"
           :steel       "6B7280"
           :silver      "9AA3B2"
           :platinum    "C8CDD8"))
