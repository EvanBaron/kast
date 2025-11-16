{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        llvm = pkgs.llvmPackages_latest;
      in
      with pkgs;
      rec {
        devShell = mkShell rec {
          buildInputs = [
            libxkbcommon
            libGL

            # WINIT_UNIX_BACKEND=wayland
            wayland

            # WINIT_UNIX_BACKEND=x11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libX11

            llvm.clang
            llvm.libclang
            vulkan-loader
            pkg-config
          ];

          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
          LIBCLANG_PATH = "${llvm.libclang.lib}/lib";

          # Critical for bindgen to find system headers
          BINDGEN_EXTRA_CLANG_ARGS =
            # From: https://github.com/NixOS/nixpkgs/blob/master/pkgs/applications/networking/browsers/firefox/common.nix
            # Set C flags for Rust's bindgen program. Unlike ordinary C
            # compilation, bindgen does not invoke $CC directly. Instead it
            # uses LLVM's libclang. To make sure all necessary flags are
            # included we need to look in a few places.
            builtins.concatStringsSep " " [
              (builtins.readFile "${stdenv.cc}/nix-support/libc-crt1-cflags")
              (builtins.readFile "${stdenv.cc}/nix-support/libc-cflags")
              (builtins.readFile "${stdenv.cc}/nix-support/cc-cflags")
              (builtins.readFile "${stdenv.cc}/nix-support/libcxx-cxxflags")
              (lib.optionalString stdenv.cc.isClang "-idirafter ${stdenv.cc.cc.lib}/lib/clang/${lib.getVersion stdenv.cc.cc}/include")
              (lib.optionalString stdenv.cc.isGNU "-isystem ${lib.getDev stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc} -isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc}/${stdenv.hostPlatform.config} -idirafter ${stdenv.cc.cc}/lib/gcc/${stdenv.hostPlatform.config}/${lib.getVersion stdenv.cc.cc}/include")
            ];
        };
      }
    );
}
