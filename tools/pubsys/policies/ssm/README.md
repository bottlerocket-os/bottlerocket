# Parameter templates

Files in this directory contain template strings that are used to generate SSM parameter names and values.
You can pass a different directory to `pubsys` to use a different set of parameters.

The directory is expected to contain a file named `defaults.toml` with a table entry per parameter, like this:

```
[[parameter]]
name = "{variant}/{arch}/{image_version}/image_id"
value = "{image_id}"
```

The `name` and `value` can contain template variables that will be replaced with information from the current build and from the AMI registered from that build.

The available variables include:
* `variant`, for example "aws-k8s-1.17"
* `arch`, for example "x86_64"
* `image_id`, for example "ami-0123456789abcdef0"
* `image_name`, for example "bottlerocket-aws-k8s-1.17-x86_64-v0.5.0-e0ddf1b"
* `image_version`, for example "0.5.0-e0ddf1b"
* `region`, for example "us-west-2"

# Overrides

You can also add or override parameters that are specific to `variant` or `arch`.
To do so, create a directory named "variant" or "arch" inside parameters directory, and create a file named after the specific variant or arch for which you want overrides.

For example, to add extra parameters just for the "aarch64" architecture, create `arch/aarch64.toml`.
Inside you can put the same types of `[[parameter]]` declarations that you see in `defaults.toml`, but they'll only be applied for `aarch64` builds.
