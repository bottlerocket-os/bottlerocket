{ lib, buildGoModule, fetchFromGitHub }:
let
  rev = "0bf29e753d6c90a04fe0f8255d345fde3684fdf2";
  snapshotDate = "20191002";
in
buildGoModule rec {
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

  amiID = "ami-0346bb6ef129f9f11";
  patches = [./eksctl-thar.patch ];
  postPatch = ''
    substituteInPlace pkg/ami/static_resolver_ami.go \
                      --subst-var amiID
  '';

  CGO_ENABLED=0;
  buildFlags = [ "-tags netgo" "-tags release"
                 "-ldflags" "-X github.com/weaveworks/eksctl/pkg/version.gitCommit=${rev} -X github.com/weaveworks/eksctl/pkg/version.builtAt=${snapshotDate} -X github.com/weaveworks/eksctl/pkg/version.gitTag=${version}" ];

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
