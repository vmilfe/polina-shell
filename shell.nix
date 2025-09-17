{ pkgs ? import <nixpkgs> {} }:

let
  libs = with pkgs; [
    libGL
    libxkbcommon
    wayland
    cargo
  ];
in
pkgs.mkShell {
  buildInputs = [ pkgs.cargo pkgs.rustc ] ++ libs;

  shellHook = ''
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath libs}"
    export WINIT_UNIX_BACKEND=x11
  '';
}