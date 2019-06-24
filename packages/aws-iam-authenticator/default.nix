{ rpmBuilder }:
rpmBuilder.mkDerivation rec {
  name = "aws-iam-authenticator";
  src = ./.;
}
