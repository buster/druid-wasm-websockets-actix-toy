#nix-channel --add https://nixos.org/channels/nixos-unstable nixpkgs
#echo "import (builtins.fetchTarball {
#      url = https://github.com/nix-community/emacs-overlay/archive/master.tar.gz;
#    })" >> $HOME/.config/nixpkgs/overlays/emacs.nix
# nix-env -iA cachix -f https://cachix.org/api/v1/install
# cachix use nix-community

{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
#    pkgs.adoptopenjdk-hotspot-bin-11
#    pkgs.kubectl
#    pkgs.maven
#    pkgs.docker-compose
#    pkgs.emacsGcc
    #pkgs.graalvm11-ce
    # keep this line if you use bash
    pkgs.bashInteractive
  ];
}
