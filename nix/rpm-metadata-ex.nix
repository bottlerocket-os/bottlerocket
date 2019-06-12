with import ./default.nix;
rec {
  specs = nixpkgs.lib.sourceFilesBySuffices ../packages [".spec" "sources"];
  # TODO: generate this list.
  #specs' = [ ../packages/bash/bash.spec ../packages/coreutils/coreutils.spec ];
  specs' = [ ../packages/bash/bash.spec ../packages/coreutils/coreutils.spec ../packages/glibc/glibc.spec ../packages/grub/grub.spec ../packages/kernel/kernel.spec ../packages/libacl/libacl.spec ../packages/libattr/libattr.spec ../packages/libcap/libcap.spec ../packages/libkmod/libkmod.spec ../packages/libxcrypt/libxcrypt.spec ../packages/ncurses/ncurses.spec ../packages/readline/readline.spec ../packages/release/release.spec ../packages/ripgrep/ripgrep.spec ../packages/rust/rust.spec ../packages/sdk/sdk.spec ../packages/signpost/signpost.spec ../packages/strace/strace.spec ../packages/systemd/systemd.spec ../packages/util-linux/util-linux.spec];
  
  specs-metadata = map (specFile:
    let
      specSources = dirOf specFile;
    in
      rpm-metadata { inherit specFile specSources; }
      ) specs';
      specs-sources = map (drv: (source-fetcher { sources = "${drv}/sources.json"; useFile = true; })) specs-metadata;
  
  # bash = rpm-metadata { specFile = ../packages/bash/bash.spec; specSources = ../packages/bash; };
  # bash-sources = (source-fetcher { sources = "${bash}/sources.json"; useFile = true; });
}

