# Secure Boot Keys for Bottlerocket

This document describes the tools available to generate the files needed for Secure Boot support in Bottlerocket.

## Background

For Secure Boot support, many different keys, certificates, and configuration files are required for building and publishing images.
The [ArchWiki guide to Secure Boot](https://wiki.archlinux.org/title/Unified_Extensible_Firmware_Interface/Secure_Boot#Using_your_own_keys) covers the purpose of most of these files, along with sample commands to generate them.
To keep build logic simple, a complete set of these files must be present for each build, and the files must follow the expected naming conventions.

Each set of files is referred to as a Secure Boot Keys ("sbkeys") profile.
The tools provided in this directory can be used to generate either a [local profile](#create-a-profile-with-local-resources) or an [AWS-based profile](#create-a-profile-with-aws-based-resources).
If your preferred solution for key management is not supported, a contribution that adds a new tool or profile type would be welcome.

To streamline the process of building Bottlerocket with Secure Boot support, a local profile will be generated automatically.
This is done to minimize costs and to avoid requiring developers to set up infrastructure for key management ahead of time.
However, because these local profiles offer direct access to private key materials, they are **strongly discouraged** for any kind of production use.

Different profiles can be [specified at build time](#specify-a-profile-at-build-time) to use a custom set of keys.

## Create a profile with local resources

The `generate-local-sbkeys` tool can be used to create a local Secure Boot Keys profile.

It uses `openssl` to generate private keys and certificate authorities (CAs), and `gpg` to create a GPG private key.
It also uses [virt-fw-vars](https://pypi.org/project/virt-firmware/) to generate EFI variable data for the edk2 variable stores used by Amazon EC2 AMIs and QEMU.

When specifying an SDK image, these dependencies are run within a container started from that image, and do not need to be installed on the host.

```shell
ARCH="$(uname -m)"
SDK_VERSION="v0.29.0"
./generate-local-sbkeys \
  --sdk-image "public.ecr.aws/bottlerocket/bottlerocket-sdk-${ARCH}:${SDK_VERSION}" \
  --output-dir "${PWD}/my-local-profile"
```

## Create a profile with AWS-based resources

The `generate-aws-sbkeys` tool can be used to create an AWS-based Secure Boot Keys profile.

It uses the AWS CLI and [aws-kms-pkcs11](https://github.com/JackOfMostTrades/aws-kms-pkcs11) to obtain certificates, and the `virt-fw-vars` tool to generate EFI variable data.
It creates an `aws-kms-pkcs11` configuration file for subsequent signing operations.

When specifying an SDK image, these dependencies are run within a container started from that image, and do not need to be installed on the host.

The tool expects four [AWS Private CAs](https://docs.aws.amazon.com/privateca/latest/userguide/PcaWelcome.html) and three [AWS KMS asymmetric keys](https://docs.aws.amazon.com/kms/latest/developerguide/concepts.html#asymmetric-keys-concept) to be available.
Note that the cost of these resources, **especially the private CAs**, is nontrivial.

Although it is possible to use the same private CA and the same KMS key for all roles, doing so would weaken the security of the implementation, and is **strongly discouraged**.

```shell
ARCH="$(uname -m)"
SDK_VERSION="v0.29.0"

# AWS Private CAs
PK_CA="arn:aws:acm-pca:us-west-2:999999999999:certificate-authority/11111111-1111-1111-1111-111111111111"
KEK_CA="arn:aws:acm-pca:us-west-2:999999999999:certificate-authority/22222222-2222-2222-2222-222222222222"
DB_CA="arn:aws:acm-pca:us-west-2:999999999999:certificate-authority/33333333-3333-3333-3333-333333333333"
VENDOR_CA="arn:aws:acm-pca:us-west-2:999999999999:certificate-authority/44444444-4444-4444-4444-444444444444"

# AWS KMS asymmetric keys
SHIM_SIGN_KEY="arn:aws:kms:us-west-2:999999999999:key/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
CODE_SIGN_KEY="arn:aws:kms:us-west-2:999999999999:key/bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"
CONFIG_SIGN_KEY="arn:aws:kms:us-west-2:999999999999:key/cccccccc-cccc-cccc-cccc-cccccccccccc"

./generate-aws-sbkeys \
  --sdk-image "public.ecr.aws/bottlerocket/bottlerocket-sdk-${ARCH}:${SDK_VERSION}" \
  --aws-region us-west-2 \
  --pk-ca "${PK_CA}" \
  --kek-ca "${KEK_CA}" \
  --db-ca "${DB_CA}" \
  --vendor-ca "${VENDOR_CA}" \
  --shim-sign-key "${SHIM_SIGN_KEY}" \
  --code-sign-key "${CODE_SIGN_KEY}" \
  --config-sign-key "${CONFIG_SIGN_KEY}" \
  --output-dir "${PWD}/my-aws-profile"
```

To generate the profile, the IAM user or role should have a policy like this:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Action": [
        "acm-pca:GetCertificate",
        "acm-pca:GetCertificateAuthorityCertificate",
        "acm-pca:IssueCertificate",
        "kms:GetPublicKey",
        "kms:Sign"
      ],
      "Effect": "Allow",
      "Resource": "*"
    }
  ]
}
```

To use the profile to sign artifacts during the build process, the IAM user or role should have a policy like this:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Action": [
        "kms:GetPublicKey",
        "kms:Sign"
      ],
      "Effect": "Allow",
      "Resource": "*"
    }
  ]
}
```

## Specify a profile at build time

To use a custom Secure Boot Keys profile, set the `BUILDSYS_SBKEYS_PROFILE` variable at build time, like this:

```shell
cargo make -e BUILDSYS_SBKEYS_PROFILE=my-custom-profile
```

Since all the files in a Secure Boot Keys profile are plain text and source-control friendly, you may wish to store them in a separate directory backed by Git or some other SCM.
To refer to profiles in a different directory, set the `BUILDSYS_SBKEYS_DIR` variable at build time, like this:

```shell
cargo make \
  -e BUILDSYS_SBKEYS_DIR="${HOME}/my-sbkeys" \
  -e BUILDSYS_SBKEYS_PROFILE=my-custom-profile
```
