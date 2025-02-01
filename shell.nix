{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
    buildInputs = [
        pkgs.gcc
        pkgs.rustup
        pkgs.llvmPackages_18.bintools
        pkgs.cmake
        pkgs.clang
        pkgs.pkg-config
        pkgs.wayland
        pkgs.glfw
        pkgs.libGL
        pkgs.xorg.libXrandr
        pkgs.xorg.libXinerama
        pkgs.xorg.libXcursor
        pkgs.xorg.libXi
        pkgs.xorg.libX11
    ];

    shellHook = ''
        export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [
            pkgs.libGL
            pkgs.xorg.libXrandr
            pkgs.xorg.libXinerama
            pkgs.xorg.libXcursor
            pkgs.xorg.libXi
            pkgs.xorg.libX11
        ]}
        export LIBCLANG_PATH=${pkgs.llvmPackages_18.libclang.lib}/lib
    '';
}