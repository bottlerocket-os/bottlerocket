## RSA-PSS

```
# Generate private key
openssl genrsa 3072 >private.pem

# Display public key
openssl pkey -pubout <private.pem

# Sign stdin in a way ring agrees with
openssl dgst -sha256 -sigopt rsa_padding_mode:pss -sigopt rsa_pss_saltlen:32 -sign private.pem
```
