```nix
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    gtk4
    pkg-config
    libadwaita.dev
    meson
    desktop-file-utils
    pkgs.gnumake
    
  ];
}
```