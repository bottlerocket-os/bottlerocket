{ docker-run }:
docker-run {
  name = "example-rpmbuild";
  image = "fedora:latest";
#  extraArgs = [ "--dns" "8.8.8.8" ];
  containerPhase = ''
  set -xe
  dnf install -y procps-ng
  hash ps tee
  ps aux | tee $out
  '';
}
