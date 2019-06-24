{ callPackage }:
let
  tharPackages = rec {
    inherit tharPackages;

    signpost = callPackage ./signpost/default.nix {};
    bash = callPackage ./bash/default.nix {};
    coreutils = callPackage ./coreutils/default.nix {};
    sdk = callPackage ./sdk/default.nix {};
    readline = callPackage ./readline/default.nix {};
    ncurses = callPackage ./ncurses/default.nix {};
    libxcrypt = callPackage ./libxcrypt/default.nix {};
    rust = callPackage ./rust/default.nix {};
    api = callPackage ./api/default.nix {};
    libacl = callPackage ./libacl/default.nix {};
    libattr = callPackage ./libattr/default.nix {};
    util-linux = callPackage ./util-linux/default.nix {};
    systemd = callPackage ./systemd/default.nix {};
    release = callPackage ./release/default.nix {};
    libkmod = callPackage ./libkmod/default.nix {};
    grub = callPackage ./grub/default.nix {};
    filesystem = callPackage ./filesystem/default.nix {};
    libcap = callPackage ./libcap/default.nix {};
    glibc = callPackage ./glibc/default.nix {};
    strace = callPackage ./strace/default.nix {};
    kernel = callPackage ./kernel/default.nix {};
    ripgrep = callPackage ./ripgrep/default.nix {};

    # Aliases
    gcc = sdk;
    kernel-headers = kernel;
  };
in
tharPackages
