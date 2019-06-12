{ stdenv }:
stdenv.mkDerivation {
  name = "fedora-base-image";
  src = (builtins.fetchurl {
    url = "https://kojipkgs.fedoraproject.org//packages/fedora-container-image/30/2/images/fedora-container-image-30-2.x86_64.tar.xz";
    sha256 = "0l4klphcvx7k8zsyjp5q0rjs2f280y279aa604z4zq9rasrii8dy";
  });
  installPhase = "ln -s $src $out";
  phases = [ "installPhase" ];
}
