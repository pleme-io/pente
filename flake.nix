{
  description = "pente — the fleet theme vocabulary. A theme is authored as a (deftheme …) tatara-lisp form over a typed Rust border and rendered to every consumer, so a palette has one source and no consumer holds a copy.";
  inputs = {
    nixpkgs = {
      follows = "substrate/nixpkgs";
    };
    crate2nix = {
      url = "github:nix-community/crate2nix";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
    substrate = {
      url = "github:pleme-io/substrate";
    };
  };
  outputs = inputs @ { self, nixpkgs, crate2nix, flake-utils, substrate, ... }:
    (import "${substrate}/lib/rust-workspace-release-flake.nix" {
      inherit nixpkgs crate2nix flake-utils;
    }) {
      toolName = "pente-forge";
      packageName = "pente-forge";
      src = self;
      repo = "pleme-io/pente";
    };
}
