{
  pkgs ? import <nixpkgs> { },
}:

let
  llvm = pkgs.llvmPackages_latest;
in
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    libxkbcommon
    libGL

    # WINIT_UNIX_BACKEND=wayland
    wayland

    # WINIT_UNIX_BACKEND=x11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    xorg.libX11
    xorg.libxcb

    llvm.clang
    llvm.libclang
    vulkan-loader
    vulkan-validation-layers
    pkg-config
  ];

  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
  LIBCLANG_PATH = "${llvm.libclang.lib}/lib";
  VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";

  # Critical for bindgen to find system headers
  BINDGEN_EXTRA_CLANG_ARGS =
    # From: https://github.com/NixOS/nixpkgs/blob/master/pkgs/applications/networking/browsers/firefox/common.nix
    # Set C flags for Rust's bindgen program. Unlike ordinary C
    # compilation, bindgen does not invoke $CC directly. Instead it
    # uses LLVM's libclang. To make sure all necessary flags are
    # included we need to look in a few places.
    builtins.concatStringsSep " " [
      (builtins.readFile "${pkgs.stdenv.cc}/nix-support/libc-crt1-cflags")
      (builtins.readFile "${pkgs.stdenv.cc}/nix-support/libc-cflags")
      (builtins.readFile "${pkgs.stdenv.cc}/nix-support/cc-cflags")
      (builtins.readFile "${pkgs.stdenv.cc}/nix-support/libcxx-cxxflags")
      (pkgs.lib.optionalString pkgs.stdenv.cc.isClang "-idirafter ${pkgs.stdenv.cc.cc.lib}/lib/clang/${pkgs.lib.getVersion pkgs.stdenv.cc.cc}/include")
      (pkgs.lib.optionalString pkgs.stdenv.cc.isGNU "-isystem ${pkgs.lib.getDev pkgs.stdenv.cc.cc}/include/c++/${pkgs.lib.getVersion pkgs.stdenv.cc.cc} -isystem ${pkgs.stdenv.cc.cc}/include/c++/${pkgs.lib.getVersion pkgs.stdenv.cc.cc}/${pkgs.stdenv.hostPlatform.config} -idirafter ${pkgs.stdenv.cc.cc}/lib/gcc/${pkgs.stdenv.hostPlatform.config}/${pkgs.lib.getVersion pkgs.stdenv.cc.cc}/include")
    ];
}
