{ lib, buildGoModule, fetchFromGitHub, go-bindata, versionExtra ? "" }:
let
  rev = "0.13.0";
  snapshotDate = "20200210";
in
buildGoModule rec {
  pname = "eksctl";
  version = "0.13.0";

  src = fetchFromGitHub {
    inherit rev;
    owner = "weaveworks";
    repo = "eksctl";
    sha256 = "13kxilsy0fdzg1phzcsxfg53flzx3xk6c5jyygggajp45aysbyra";
  };
  modSha256 = "0g5alqhwna9sd6dp50waqa87af2z3n5pj5mwnb9i2y65g2kclaha";
  subPackages = [ "cmd/eksctl" ];

  CGO_ENABLED=0;
  buildFlags = [ "-tags netgo" "-tags release" ];

  patches = [./eksctl-thar.patch ./update-cni-1.6.0-rc6.patch ];

  postConfigure = let
    tag = "${version}${lib.optionalString (versionExtra != "") "/${versionExtra}"}";
  in ''
    echo "updating go-bindata assets"
    pushd pkg/addons &>/dev/null
    ${go-bindata}/bin/go-bindata -pkg addons -prefix assets -nometadata -o assets.go assets
    cd default &>/dev/null
    ${go-bindata}/bin/go-bindata -pkg defaultaddons -prefix assets -nometadata -o assets.go assets
    popd

    # Hardcode source data
    cat > pkg/version/release.go <<EOF
    // +build release

    package version

    var builtAt = "${snapshotDate}"
    var gitCommit = "${rev}"
    var gitTag = "${tag}"
    EOF
  '';

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
