# what is updog

not much what's up with you

## no really

The Updog client provides an interface to a TUF repository and prepares for, downloads, and applies updates to the Bottlerocket instance. Updog can be called manually, but will more commonly be called automatically by some cluster orchestrator. For usage run `updog --help`.

## Quick reference

### Check for the most recent update
```
# updog check-update
aws-k8s-1.15 0.1.4 (v0.0)
```

### List all available updates, including older versions
```
# updog check-update --all
aws-k8s-1.15 0.1.4 (v0.0)
aws-k8s-1.15 0.1.2 (v0.0)
aws-k8s-1.15 0.1.1 (v0.0)
```

### Specify JSON output
```
# updog check-update --json
[{"variant":"aws-k8s-1.15","arch":"x86_64","version":"0.1.4","max_version":"0.1.4","waves":{"512":"2019-10-03T20:45:52Z","1024":"2019-10-03T21:00:52Z","1536":"2019-10-03T22:00:52Z","2048":"2019-10-03T23:00:52Z"},"images":{"boot":"bottlerocket-x86_64-aws-k8s-1.15-v0.1.4-boot.ext4.lz4","root":"bottlerocket-x86_64-aws-k8s-1.15-v0.1.4-root.ext4.lz4","hash":"bottlerocket-x86_64-aws-k8s-1.15-v0.1.4-root.verity.lz4"}}]
```

### Try to update with wave information
```
# updog update
Update available at 2019-10-03 21:24:00 UTC
```
Once timestamp has passed:
```
# updog update --timestamp 2019-10-03T21:24:00+00:00
Starting update to 0.1.4
Update applied: aws-k8s-1.15 0.1.4
```

### Force an immediate update, ignoring wave limits
```
# updog update --now
Starting update to 0.1.4
** Updating immediately **
Update applied: aws-k8s-1.15 0.1.4
```

## Proxy Support

The `network.https-proxy` and `network.no-proxy` settings are taken from updog's config file.
These will override the environment variables `HTTPS_PROXY` and `NO_PROXY`.
