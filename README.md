```nix
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    gtk4
    pkg-config
    libadwaita       
    meson
    desktop-file-utils
    gnumake         
    cargo
    rustc
    rustfmt
    clippy
    rust-analyzer
  ];
}
```