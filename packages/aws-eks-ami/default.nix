{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "aws-eks-ami";
  src = ./.;
}
