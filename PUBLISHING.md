# Publishing Bottlerocket

This guide will walk you through deploying a Bottlerocket image, and if desired, sharing it with others.
It currently focuses on deploying to AWS, though the tooling is built to support other platforms in the future.

Remember to look at the [TRADEMARKS](TRADEMARKS.md) guide to understand naming concerns.
You can pass `-e BUILDSYS_NAME=my-name` to `cargo make` commands to change the default "short" name, which is used in file and AMI names.
You can pass `-e BUILDSYS_PRETTY_NAME="My Name"` to `cargo make` commands to change the default "pretty" name, which is used in the os-release file and some menus.

We'll assume you've been through the [BUILDING](BUILDING.md) guide to make an image.

### Configuring the publishing process

The publishing process uses a configuration file called `Infra.toml`.
The relevant sections of this file will be introduced as needed below.
You can also see an [example file](tools/pubsys/Infra.toml.example) where each section is commented.

When you make your own `Infra.toml`, you put it in the root of the Bottlerocket code repo, wherever you have it checked out.
(If you want to keep it elsewhere, you can pass `-e "PUBLISH_INFRA_CONFIG_PATH=/my/path"` to subsequent `cargo make` commands.)

Note: several commands work with AWS services, so there's some shared configuration related to AWS accounts and AWS IAM roles.
For example, you can specify a role to assume before any API calls are made, and a role to assume before any API calls in a specific region.
This can be useful if you want to use roles to control access to the accounts that own AMIs, for example.
See the commented [example Infra.toml](tools/pubsys/Infra.toml.example) for details.

### Variants and architectures

If you [built your image](BUILDING.md) for a different variant or architecture, you can pass the same variant and architecture arguments to any of the `cargo make` commands in this document.
For example, if you built your image like this:

```shell
cargo make -e BUILDSYS_VARIANT=my-variant -e BUILDSYS_ARCH=my-arch
```

...then you can then build a repo for it like this:

```shell
cargo make repo -e BUILDSYS_VARIANT=my-variant -e BUILDSYS_ARCH=my-arch
```

## Register an AMI

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

## Granting access to your AMI

If you use different accounts to make and test your AMIs, you can grant access to specific accounts like this:

```shell
cargo make grant-ami -e GRANT_TO_USERS=0123456789,9876543210
```

(Later, if you need to revoke access, you can do this:)

```shell
cargo make revoke-ami -e REVOKE_FROM_USERS=0123456789,9876543210
```

> Note: similar to `cargo make ami`, you can specify `PUBLISH_REGIONS` on the command line if you don't want to make an `Infra.toml` config.

## Build a repo

__NOTE: If you intend to replace hosts rather than updating them, you don't need to build an update repository.__

Bottlerocket uses [TUF repositories](https://theupdateframework.io/overview/) to make system updates available to hosts.
You can read more about how Bottlerocket uses TUF in the [updater README](sources/updater/README.md#tuf-and-tough).

Initially, the repo will only contain the image you just built.
Later, when you build updates, you can [add them to the repo](#configuring-your-repo-location), which allows your hosts to update to new versions.
(If you don't have an `Infra.toml` file, it will always try to build a brand new repo.)

### Build process

To build a repo, run:

```shell
cargo make repo
```

#### Picking a release time

If you're preparing the release of a new version in advance (see [waves](#waves) for why you may want to) you can specify the start time for the release.
You'll need the time in ISO 8601 format.
You can use the `date` command to get the formatted time using a simple description of your desired start.
For example, if you want your release to start at 10:00 AM on Monday:

```shell
RELEASE_START_TIME="$(date '+%Y-%m-%dT%H:%M:%S%:z' -d 'Monday 10am')"
```

Now we can create the repo using that time:

```shell
cargo make repo -e "RELEASE_START_TIME=${RELEASE_START_TIME}"
```

### Roles and keys

#### Background on roles and keys

TUF repos use [signed metadata](https://theupdateframework.io/metadata/) to ensure the repo content is secure and consistent.
Bottlerocket images contain a signed root role that verifies the data in the update repo they talk to.

If you run the `cargo make repo` command above without any configuration, it will generate a root role file and a signing key for you.

The generated role and key are functional, but a bit basic.
There's only a single key, and a "signing threshold" of 1, meaning only 1 key needs to sign replacement keys.
For production use, you should consider having multiple root keys with a higher signing threshold.
The benefit is that if someone compromises a single root key, TUF libraries won't trust any new keys they try to issue.

It's also a good idea to keep your key somewhere safer than your local disk.
This helps guard against loss of the key, which would leave you unable to update your repo.
We currently support storing keys in local files, in [AWS SSM Parameters](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html), and in [AWS KMS](https://aws.amazon.com/kms/).
SSM supports encrypted "SecureString" parameters for cases like this, and you can upload an existing private key into a parameter.
KMS is even stronger in that private keys can never be uploaded or read - they're held in secure hardware.

Another improvement is to separate your root key from your "publication" key, where the publication key controls the snapshot, targets, and timestamp roles.
Those three roles are updated a lot more frequently.
The benefit is that even if the publication key is compromised, you still control the root key, and can replace the publication key.

To use a separate publication key, you can generate two keys using [tuftool](https://github.com/awslabs/tough/tree/develop/tuftool).
Assuming you have a root.json from `tuftool root init`, you can create keys like this:

```shell
tuftool root gen-rsa-key /path/to/root.json /path/to/my-new-root-key.pem --role root
tuftool root gen-rsa-key /path/to/root.json /path/to/my-new-publication-key.pem --role snapshot --role targets --role timestamp
```

If you're using keys in SSM or KMS, then you can add them to your root role with a similar command.
For example, with a KMS key, instead of `gen-rsa-key` you'd run `add-key` like this:

```shell
tuftool root add-key /path/to/root.json aws-kms:///abc-def-123 --role root
tuftool root add-key /path/to/root.json aws-kms:///456-cba-fed --role snapshot --role targets --role timestamp
```

#### Role and key configuration

You can specify your own root role and your own key in `Infra.toml`.
Root roles and keys are associated with a specific named repo.
The publishing system assumes a repo named "default", so it's easiest to get started by using that name.
(You can also pass `-e PUBLISH_REPO=myrepo` to `cargo make` commands to use a different name.)

Here's an example repo configuration in `Infra.toml`:

```toml
[repo.default]
root_role_url = "https://example.com/root.json"
root_role_sha512 = "0123456789abcdef"
signing_keys = { file = { path = "/home/user/key.pem" } }
```

If you have your own root role, you specify it by URL; this can be a `file://` URL for a local file.
You also specify the SHA512 checksum, to confirm that the file is the one you expect, in case we're downloading it from a remote URL.
There's nothing secret in a root role file, so if you have a way of storing it remotely, a URL can be more convenient.

The `signing_keys` portion above references a local file path.
If you want to use an SSM or KMS key, you'd write it like this, instead:

```
signing_keys = { kms = { key_id = "abc-def-123" } }
```

...or...

```
signing_keys = { ssm = { parameter = "/my/parameter" } }
```

### Repo location

#### Uploading your repo

Your repo needs to be accessible to your hosts by URL.
One good place to store repos is S3; this is how Bottlerocket's official repos are stored.
(If you want, you can put a CloudFront distribution on top of this to make it accessible even more quickly around the world.)
You can also store your repo behind any HTTP server; the key part is that the repo is accessible from your host.
This could mean it's publicly accessible, or only accessible inside a VPC, or something similar.

Let's assume you're using an S3 bucket.
You just need to sync the built repo, like this.
(If you're using a repo other than `default`, make sure you change the repo name.)

```shell
aws s3 sync build/repos/default/latest/ s3://my-bucket/
```

This syncs the metadata and targets directories of the repo into the root of your bucket.
You can also sync to a subdirectory of your bucket if desired, for example if you use the bucket for other purposes.
Just make sure you include that subdirectory in the URL in the next step.

> Note: for production repos, it's safer to sync the targets directory before the metadata directory so that clients aren't pointed to targets they can't download yet.

#### Configuring your repo location

After your repo is uploaded, you can add the location into the repo configuration in your `Infra.toml`.
This will allow you to use `cargo make repo` to update your existing repo in the future, rather than creating a new one from scratch every time.
This is important so that your hosts can see all available updates in the repo, not just the latest one.

Inside the repo section of your `Infra.toml` (for example, underneath `[repo.default]`) you'd add something like this:

```toml
metadata_base_url = "https://example.com/"
targets_url = "https://example.com/targets/"
```

(You can use a `file://` URL if you want to update a repo based on one you keep locally.)

The variant and architecture are automatically added onto the metadata URL, matching the format of the directories inside `build/repos/default/latest`.
(The targets directories is shared for all variants and architectures, since target files are prefixed with a checksum.)

### Using your repo from a Bottlerocket host

By default, Bottlerocket hosts talk to the project's official repos.
There are two ways to point your hosts at your own repo - at build time or at run time.

If you're maintaining your own fork of Bottlerocket, you'd probably want to change the settings at build time, so you don't have to change settings for every host you launch.
If you're just running a few hosts, or don't want to maintain a fork, then it's easier to change settings at run time.

To change your repo URLs at build time, you would change the `settings.updates.targets-base-url` and `metadata.settings.updates.metadata-base-url.template` settings.

The default settings are defined in TOML files.
First, open the directory for your variant under [sources/models/src/](sources/models/src/).
Then, open the `defaults.d` directory.
Here, you can have any number of TOML files, or symlinks to shared TOML files, that define your default settings.
Later files override earlier ones.
For an example, take a look at the [aws-ecs-1 defaults](sources/models/src/aws-ecs-1/defaults.d/).

These default settings will be applied to your hosts at startup, meaning any host you run would already know to look at your repo.
(You'll probably want to commit your changes into your fork of the repo; we're working on ways of making it easier to maintain your own model and settings without a fork.)

The easiest way to change your repo URLs at run time is to include the settings changes in user data.
This method is covered [in README](README.md#using-user-data).
For example, if you built the `aws-k8s-1.17` variant for `x86_64` and uploaded to the public S3 bucket `my-bucket`, your URLs could look like:

```toml
[settings.updates]
targets-base-url = "https://my-bucket.s3-us-west-2.amazonaws.com/targets/"
metadata-base-url = "https://my-bucket.s3-us-west-2.amazonaws.com/aws-k8s-1.17/x86_64/"
```

### Waves

When you release a new version, you may want to make your update available to a small number of hosts in the beginning, then gradually expand.
This can help mitigate the risk of the change and give you more time to detect issues before they're widespread.

The Bottlerocket update system uses the concept of 'waves' of updates.
For example, you can say that you want:
* one hour before updates start, so you can prepare
* 1% of hosts to get the update within 4 hours
* 5% of hosts to get the update within 1 day
* 15% of hosts to get the update within 2 days
* 40% of hosts to get the update within 4 days
* 60% of hosts to get the update within 5 days
* 90% of hosts to get the update within 6 days
* 100% of hosts to get the update after 6 days

This provides a gradual ramp-up so you can watch the status of your deployment more easily.
And, in fact, this is the default wave policy!

The policy above is defined in [default-waves](sources/updater/waves/default-waves.toml).
There's also an [accelerated schedule](sources/updater/waves/accelerated-waves.toml) for more urgent deployments, and an ["oh no" schedule](sources/updater/waves/ohno.toml) for emergencies.

If you want to use a different policy, pass `-e PUBLISH_WAVE_POLICY_PATH=sources/updater/waves/chosen-policy.toml` when building your repo.
For example, to use the accelerated schedule:

```shell
cargo make repo -e PUBLISH_WAVE_POLICY_PATH=sources/updater/waves/accelerated-waves.toml
```

To learn more about waves, check out the [README](sources/updater/waves).

### Expiration policy

Each piece of signed metadata in a TUF repo expires after a specific length of time, meaning that repos need to re-signed regularly.
This lets users know that the repo has been verified recently by the owner.

The [default policy](tools/pubsys/policies/repo-expiration/2w-2w-1w.toml) sets the timestamp expiration relatively short, [as recommended by TUF](https://theupdateframework.io/metadata/#timestamp-metadata-timestampjson), with the snapshot and targets expirations a bit longer.
If you want to use different expiration policy, you can copy and modify the existing policy, then point to your file like this:

```shell
cargo make repo -e PUBLISH_EXPIRATION_POLICY_PATH=/my/policy/path
```

**Note:** remember to update your repo before the expiration date.
If you forget, your hosts won't be able to talk to the repo until you update it.
(Don't worry, they're not lost forever.)

Currently, to refresh an existing repo, you would use the [tuftool update](https://github.com/awslabs/tough/tree/develop/tuftool) command without specifying any new targets.
We're working on ways to make this easier, and integrated into the `cargo make` system.

## Making your AMIs discoverable with SSM parameters

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
/your/prefix/here/aws-k8s-1.17/x86_64/1.0.1-dafe3b16/image_id
```

Once you're satisfied with your image and parameters, you can promote the parameters to simpler names (for example, "latest") using the [instructions below](#promoting-ssm-parameters).

Note: if you want to customize the SSM parameters that get set, you can copy and modify the existing template file, then point to your file like this:

```shell
cargo make ssm -e PUBLISH_SSM_TEMPLATES_PATH=/my/template/path
```

## Making your AMIs public

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

## Promoting SSM parameters

[Above](#making-your-amis-discoverable-with-ssm-parameters), we set SSM parameters based on our AMIs.
The SSM parameter names include version numbers, which is handy for testing, but makes them hard to find.
Once we're satisfied, we can promote the SSM parameters to simpler names.

```shell
cargo make promote-ssm -e SSM_TARGET=latest
```

This will copy the fully versioned parameter from earlier, something like:

```
/your/prefix/here/aws-k8s-1.17/x86_64/1.0.1-dafe3b16/image_id
```

...to a simpler parameter name:
```
/your/prefix/here/aws-k8s-1.17/x86_64/latest/image_id
```

You can then use this parameter name to get the latest AMI ID.

> Note: if you use a custom parameter template, you need to have an `{image_version}` component in the parameter name for promotion to work.
> The `SSM_TARGET` you specify above becomes the `image_version` in the template.
