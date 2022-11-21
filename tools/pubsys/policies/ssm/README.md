# Parameter templates

Files in this directory contain template strings that are used to generate SSM parameter names and values.
You can pass a different directory to `pubsys` to use a different set of parameters.

The directory is expected to contain a file named `defaults.toml` with a table entry per parameter, like this:

```toml
[[parameter]]
name = "{variant}/{arch}/{image_version}/image_id"
value = "{image_id}"
```

The `name` and `value` can contain template variables that will be replaced with information from the current build and from the AMI registered from that build.

The available variables include:
* `variant`, for example "aws-ecs-1"
* `arch`, for example "x86_64" or "arm64".
  * Note: "amd64" and "aarch64" are mapped to "x86_64" and "arm64", respectively, to match the names used by EC2.
* `image_id`, for example "ami-0123456789abcdef0"
* `image_name`, for example "bottlerocket-aws-ecs-1-x86_64-v0.5.0-e0ddf1b"
* `image_version`, for example "0.5.0-e0ddf1b"
* `region`, for example "us-west-2"

# Conditional parameters

You can also list parameters that only apply to specific variants or architectures.
To do so, add `variant` or `arch` keys (or both) to your parameter definition.
The parameter will only be populated if the current `variant` or `arch` matches one of the values in the list.
(If both `variant` and `arch` are listed, the build must match an entry from both lists.)

For example, to add an extra parameter that's only set for "aarch64" builds of the "aws-ecs-1" variant:
```toml
[[parameter]]
arch = ["aarch64"]
variant = ["aws-ecs-1"]
name = "/a/special/aarch64/ecs/parameter"
value = "{image_name}"
```
