{ pkgs ? import <nixpkgs> {} }:

let
  nova-src = builtins.fetchTarball {
    url = "https://github.com/pyrotek45/nova-lang/archive/main.tar.gz";
  };
in
pkgs.rustPlatform.buildRustPackage {
  pname = "nova";
  version = "0.1.0";

  src = nova-src;

  cargoLock.lockFile = "${nova-src}/Cargo.lock";

  nativeBuildInputs = [
    pkgs.cmake
    pkgs.clang
    pkgs.pkg-config
    pkgs.llvmPackages_18.bintools
  ];

  buildInputs = [
    pkgs.libGL
    pkgs.xorg.libX11
    pkgs.xorg.libXrandr
    pkgs.xorg.libXinerama
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.wayland
    pkgs.glfw
  ];

  LIBCLANG_PATH = "${pkgs.llvmPackages_18.libclang.lib}/lib";

  meta = with pkgs.lib; {
    description = "The Nova programming language";
    homepage = "https://github.com/pyrotek45/nova-lang";
    license = licenses.mit;
    mainProgram = "nova";
  };
}
