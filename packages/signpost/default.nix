{ rpmBuilder, lib, fetchCargo, glibc, rust }:
let
  name = "signpost";
  project = ../../workspaces/signpost;
  cargo-toml = /. + project + /Cargo.toml;
  cargo-lock = /. + project + /Cargo.lock;
  cargo-vendor = (fetchCargo { inherit name cargo-toml cargo-lock; });
in
rpmBuilder.mkDerivation rec {
  inherit name;
  src = lib.cleanSourceWith { filter = (name: type: let baseName = baseNameOf (toString name); in
                                                    name != "default.nix");
                              src = ./.; };
  preRpmbuildCommands = ''
  tar -C ${project} -cf SOURCES/signpost.crate ./
  for d in ${cargo-vendor}/*; do
    tar -C $d -cf SOURCES/$(basename d).crate ./
  done
  '';
}
