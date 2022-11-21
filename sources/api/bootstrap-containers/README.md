# bootstrap-containers

Current version: 0.1.0

## Bootstrap containers

bootstrap-containers ensures that bootstrap containers are executed as defined in the system settings

It queries the API for their settings, then configures the system by:

* creating a user-data file in the host container's persistent storage area, if a base64-encoded
  user-data setting is set for the host container.  (The decoded contents are available to the
  container at /.bottlerocket/bootstrap-containers/<name>/user-data)
* creating an environment file used by a bootstrap-container-specific instance of a systemd service
* creating a systemd drop-in configuration file used by a bootstrap-container-specific
instance of a systemd service
* ensuring that the bootstrap container's systemd service is enabled/disabled for the next boot

## Examples
Given a bootstrap container called `bear` with the following configuration:

```toml
[settings.bootstrap-containers.bear]
source="<SOURCE>"
mode="once"
user-data="ypXCt82h4bSlwrfKlA=="
```

Where `<SOURCE>`, is the url of an image with the following definition:

```Dockerfile
FROM alpine
ADD bootstrap-script /
RUN chmod +x /bootstrap-script
ENTRYPOINT ["sh", "bootstrap-script"]
```

And `bootstrap-script` as:

```shell
#!/usr/bin/env sh
# We'll read some data to be written out from given user-data.
USER_DATA_DIR=/.bottlerocket/bootstrap-containers/current
# This is the in-container view of where the host's `/var` can be accessed.
HOST_VAR_DIR=/.bottlerocket/rootfs/var
# The directory that'll be created by this bootstrap container
MY_HOST_DIR=$HOST_VAR_DIR/lib/my_directory
# Create it!
mkdir -p "$MY_HOST_DIR"
# Write the user-data to stdout (to the journal) and to our new path:
tee /dev/stdout "$MY_HOST_DIR/bear.txt" < "$USER_DATA_DIR/user-data"
# The bootstrap container can set the permissions which are seen by the host:
chmod -R o+r "$MY_HOST_DIR"
chown -R 1000:1000 "$MY_HOST_DIR"
# Bootstrap containers *must* finish before boot continues.
#
# With this, the boot process will be delayed 120 seconds. You can check the
# status of `preconfigured.target` and `bootstrap-containers@bear` to see
# that this sleep kept the system from starting up the apiserver.
#
# From the admin container:
#
# systemctl status preconfigured.target bootstrap-containers@bear
sleep 120
```

You should see a new directory under `/var/lib` called `my_directory`, a file in that
directory called `bear.txt` and the following command should show `ʕ·͡ᴥ·ʔ` in the bootstrap
containers logs:

```shell
journalctl -u bootstrap-containers@bear.service
```

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
