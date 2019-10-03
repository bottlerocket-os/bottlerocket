{ lib, buildGoModule, fetchFromGitHub }:
{ amiID, versionExtra ? "" }:
let
  rev = "0bf29e753d6c90a04fe0f8255d345fde3684fdf2";
  snapshotDate = "20191002";
in
buildGoModule rec {
  inherit amiID;
  
  pname = "eksctl";
  version = "0.6.0";

  src = fetchFromGitHub {
    inherit rev;
    owner = "weaveworks";
    repo = "eksctl";
    sha256 = "0y9l5fy4ld2sch4g1h8lkvxbw7vi3231fz5lvwpnwas6bwvcm3v8";
  };
  modSha256 = "19my0xfssgki18syqfwbd2n8iasajy4zg0jblb0pg35lh145zz3a";
  subPackages = [ "cmd/eksctl" ];

  patches = [./eksctl-thar.patch ];
  postPatch = let
    tag = "${version}${lib.optionalString (versionExtra != "") "/${versionExtra}"}";
  in ''
    substituteInPlace pkg/ami/static_resolver_ami.go \
                      --subst-var amiID

    cat > pkg/version/release.go <<EOF
    // +build release

    package version

    // Values of builtAt and gitCommit will be set by the linker.
    var builtAt = "${snapshotDate}"
    var gitCommit = "${rev}"
    var gitTag = "${tag}"
    EOF
  '';

  CGO_ENABLED=0;
  buildFlags = [ "-tags netgo" "-tags release" ];

  postInstall =
  ''
    mkdir -p "$out/share/"{bash-completion/completions,zsh/site-functions}

    $out/bin/eksctl completion bash > "$out/share/bash-completion/completions/eksctl"
    $out/bin/eksctl completion zsh > "$out/share/zsh/site-functions/_eksctl"

    ln -s $out/bin/eksctl $out/bin/teksctl
  '';

  meta = with lib; {
    description = "A CLI for Amazon EKS";
    homepage = "https://github.com/weaveworks/eksctl";
    license = licenses.asl20;
    platforms = platforms.all;
    maintainers = with maintainers; [ xrelkd ];
  };
}
