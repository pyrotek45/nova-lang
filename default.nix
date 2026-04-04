{ pkgs ? import <nixpkgs> {} }:

let
  nova-src = builtins.fetchTarball {
    url = "https://github.com/pyrotek45/nova-lang/archive/main.tar.gz";
  };

  runtimeLibs = [
    pkgs.libGL
    pkgs.xorg.libX11
    pkgs.xorg.libXrandr
    pkgs.xorg.libXinerama
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.wayland
    pkgs.glfw
  ];
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
    pkgs.makeWrapper
  ];

  buildInputs = runtimeLibs;

  LIBCLANG_PATH = "${pkgs.llvmPackages_18.libclang.lib}/lib";

  postFixup = ''
    wrapProgram $out/bin/nova \
      --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath runtimeLibs}"
  '';

  meta = with pkgs.lib; {
    description = "The Nova programming language";
    homepage = "https://github.com/pyrotek45/nova-lang";
    license = licenses.mit;
    mainProgram = "nova";
  };
}
