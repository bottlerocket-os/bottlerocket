{ lib, buildGoModule, fetchFromGitHub, versionExtra ? "" }:
let
  rev = "f683a9daf8b4bdeb6e9f287d08b1eaa411eb294c";
  snapshotDate = "20191125";
in
buildGoModule rec {
  pname = "eksctl";
  version = "0.10.2";

  src = fetchFromGitHub {
    inherit rev;
    owner = "weaveworks";
    repo = "eksctl";
    sha256 = "1rqfxklngw0qkbasqjr9hxnhlh40chx6dv1m5d2xa0hq9h6nxjy2";
  };
  modSha256 = "1s42pdnibginjbss21fz1brdn4z6wdh8d4kxc2jwd4qbpb4lxwic";
  subPackages = [ "cmd/eksctl" ];


  patches = [./eksctl-thar.patch ];
  postPatch = let
    tag = "${version}${lib.optionalString (versionExtra != "") "/${versionExtra}"}";
  in ''
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
