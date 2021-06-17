# Publishing Bottlerocket on AWS

This guide will walk you through some AWS-specific details around publishing an AMI, granting access to said AMI, as well as making it easy for others to find your AMI via SSM parameters.

### Register an AMI

The [BUILDING](BUILDING.md#register-an-ami) guide covers the process of making an AMI, and has you specify `PUBLISH_REGIONS` to decide where the AMI will live.
You can also specify this in your `Infra.toml` file:

```toml
[aws]
regions = ["us-west-2", "us-east-1", "us-east-2"]
```

If you specify multiple regions, an AMI will be registered in the first region and then copied to the other regions.

After putting this in `Infra.toml`, you can make an AMI more easily:

```shell
cargo make ami
```

If you want to change the name or description of your AMI, you can add on `-e PUBLISH_AMI_NAME=my-name` or `-e PUBLISH_AMI_DESCRIPTION=my-desc`.

> Note: the AMI registration process creates a JSON file describing the AMIs in a directory under `build/images/`.
> This file is used by the steps below when granting access to the AMIs or setting parameters in SSM.

### Granting access to your AMI

If you use different accounts to make and test your AMIs, you can grant access to specific accounts like this:

```shell
cargo make grant-ami -e GRANT_TO_USERS=0123456789,9876543210
```

(Later, if you need to revoke access, you can do this:)

```shell
cargo make revoke-ami -e REVOKE_FROM_USERS=0123456789,9876543210
```

> Note: similar to `cargo make ami`, you can specify `PUBLISH_REGIONS` on the command line if you don't want to make an `Infra.toml` config.

### Making your AMIs discoverable with SSM parameters

After you've made AMIs and a repo, you may want to make it easier to find your AMIs, particularly as you make new versions over time.

One way to do this is to store the AMI IDs in [AWS SSM Parameters](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html).
These are simple names like `/my/ami/id` that you can use in many places instead of specific AMI IDs.
For example, you can launch EC2 instances using [RunInstances](https://docs.aws.amazon.com/systems-manager/latest/userguide/parameter-store-ec2-aliases.html) or [in a CloudFormation stack](https://aws.amazon.com/blogs/mt/integrating-aws-cloudformation-with-aws-systems-manager-parameter-store/) using a parameter name rather than an AMI ID.
You can also use the same parameter names across regions, so you don't have to deal with region-specific AMI IDs.

> Note: SSM parameters are private to your account.
> They let you use consistent names instead of tracking AMI IDs, but they don't currently let you share with other accounts.

The `cargo make ssm` task can set SSM parameters based on the AMIs you built [above](#register-an-ami).
For this to work, you have to specify a parameter prefix in your `Infra.toml`.
This setting lives in the same `[aws]` section you used above to list the regions where you want to register AMIs.
(The same region list will be used to determine where to publish SSM parameters.)

Here's an example configuration for regions and the SSM prefix:

```toml
[aws]
regions = ["us-west-2", "us-east-1", "us-east-2"]
ssm_prefix = "/your/prefix/here"
```

This prefix forms the start of the name of each SSM parameter we set.
The rest of the name comes from parameter templates.

Parameter templates determine the name and value of each parameter we want to set for each AMI we've built.
The [default template](tools/pubsys/policies/ssm/defaults.toml) creates parameters that let users find the AMI ID and the image version for each of your AMIs.
The templates have access to the name of the current variant, architecture, etc., so they can create unique parameter names for each build.
For more information on how templates work, check out [their documentation](tools/pubsys/policies/ssm/).

If you're happy with the default template, you can set SSM parameters like this:

```shell
cargo make ssm
```

This will create versioned parameters, meaning that the parameter name has the image version in it.
This isn't very discoverable yet, but it's useful for testing.

As an example, a parameter might look like this:

```
/your/prefix/here/aws-k8s-1.19/x86_64/1.0.1-dafe3b16/image_id
```

Once you're satisfied with your image and parameters, you can promote the parameters to simpler names (for example, "latest") using the [instructions below](#promoting-ssm-parameters).

Note: if you want to customize the SSM parameters that get set, you can copy and modify the existing template file, then point to your file like this:

```shell
cargo make ssm -e PUBLISH_SSM_TEMPLATES_PATH=/my/template/path
```

### Making your AMIs public

We talked about [granting AMI access](#granting-access-to-your-ami) to specific AWS accounts.
This is useful for testing, and for sharing private AMIs with specific accounts.

If you want to make your AMIs public to the world, there's a shortcut:

```
cargo make ami-public
```

(Later, if you need to make the AMIs private again, you can do this.
 The AMIs will then only be accessible to account IDs you've specifically granted.)

```shell
cargo make ami-private
```

### Promoting SSM parameters

[Above](#making-your-amis-discoverable-with-ssm-parameters), we set SSM parameters based on our AMIs.
The SSM parameter names include version numbers, which is handy for testing, but makes them hard to find.
Once we're satisfied, we can promote the SSM parameters to simpler names.

```shell
cargo make promote-ssm -e SSM_TARGET=latest
```

This will copy the fully versioned parameter from earlier, something like:

```
/your/prefix/here/aws-k8s-1.19/x86_64/1.0.1-dafe3b16/image_id
```

...to a simpler parameter name:
```
/your/prefix/here/aws-k8s-1.19/x86_64/latest/image_id
```

You can then use this parameter name to get the latest AMI ID.

> Note: if you use a custom parameter template, you need to have an `{image_version}` component in the parameter name for promotion to work.
> The `SSM_TARGET` you specify above becomes the `image_version` in the template.
