# Bottlerocket update infrastructure
This document describes the Bottlerocket update system and its components, namely;

- [tough](#tuf-and-tough): implementation of "The Update Framework" (TUF)
- [Bottlerocket update API](#update-api): allows for checking and starting system updates from TUF repo
- [apiclient](../api/apiclient/README.md): automates interactions with the update API
- [updog](#whats-updog): low-level client that interfaces with a TUF repository to find and apply updates
- [signpost](#signpost): helper tool to update partition priority flags
- [Bottlerocket update operator (brupop)](https://github.com/bottlerocket-os/bottlerocket-update-operator): an optional component that coordinates node updates with the rest of the Kubernetes cluster
- [Bottlerocket ECS updater](https://github.com/bottlerocket-os/bottlerocket-ecs-updater): an optional component that coordinates container instance updates with the rest of the ECS cluster

![Update overview](update-system.png)
## TUF and tough
A TUF repository is a collection of metadata files and 'target' files that clients can download.
In Bottlerocket there are four metadata files used to establish trusted updates: root, timestamp, snapshot, and targets.
Each metadata file is individually signed and serves a distinct purpose.

The root.json file begins the chain of trust for working with a TUF repository.
It lists the keys that a Bottlerocket instance trusts in order to verify the rest of the metadata.
The root.json file is part of the Bottlerocket image, but can be updated from the TUF repo assuming the new root.json is signed by a certain number of keys from the old.
In the Bottlerocket model multiple keys are used to sign root.json, and the loss of some amount of keys under the threshold does not prevent updating to a new root.json that contains new trusted keys.

The timestamp.json file contains the hash of the current snapshot.json file and is frequently re-signed to prevent the use of out-of-date metadata.

The snapshot.json file lists the current versions of all other metadata files in the TUF repository, aside from timestamp.json.
Once verified by timestamp.json the snapshot file ensures the client only sees the most up-to-date versions of root.json and targets.json.

The targets.json file lists all the available 'target' files in the TUF repository and their hashes.
For Bottlerocket this includes a 'manifest.json' file and any update images or migration files that have been made available.
(For more information on migrations see [migration](../api/migration))

Update metadata and files can be found by requesting and verifying these metadata files in order, and then requesting the manifest.json target which describes all available updates.
Any file listed in the manifest is also a TUF 'target' listed in targets.json and can only be downloaded via the TUF repository, preventing the client from downloading untrusted data.

## What's Updog
[Updog](updog/) is a low-level update client, used behind the scenes by the update API, that interacts with a 'The Update Framework' (TUF) repository to download and write updates to a Bottlerocket partition.
Updog will parse the manifest.json file from the TUF repository and will update to a new image if the following criteria are satisfied:

(Most users should use [apiclient](../api/apiclient/README.md) to control updates, rather than using updog directly.)

### Version & Variant
By default Updog only considers updates resulting in a version increase; downgrades are possible by using the `--image` option to force a specific version.
Updog will respect the `max_version` field in the update manifest and refuse to update beyond it.
Updog also considers the Bottlerocket "variant" of its current image and will not download updates for a different variant.

Updog will ensure that appropriate migration files are available to safely transition to the new version and back.

### Update wave
Updates may include "wave" information which provides a way for updates to be scheduled over time for groups of Bottlerocket hosts.
Updog will find the update wave the host belongs to and calculate its time position within the wave based on its `settings.updates.seed` value.
If the calculated time has not passed, Updog will not report an update as being available.

Assuming all the requirements are met, Updog requests the update images from the TUF repository and writes them to the "inactive" partition.

For more information on what's Updog see [Updog](updog/).
For more information about update waves see [Waves](waves/).

## Signpost
Once an update has been successfully written to the inactive partition, Updog calls the Signpost utility.
This updates the priority bits in the GUID partition table of each partition and swaps the "active" and "inactive" partitions.
For more information see [Signpost](signpost/)

## Update API
The [Bottlerocket API](../../README.md#api) allows you to update and reboot your host.  You can change [settings](../../README.md#updates-settings) to control which updates will be selected.

[thar-be-updates](../api/thar-be-updates) is the component driving the update actions in the background.

In general, the process of using the update API looks like this:
![Update API overview](../api/update_api.png)

You refresh the list of known updates, then apply one to the system.
Calls to `/updates/status` will tell you the current state and give more details on any errors.

`apiclient` understands this workflow and automates the calls for most use cases.
See the [apiclient README](../api/apiclient/README.md) for details.

### Walkthrough

If you don't want to use the simpler update mode available in [apiclient](../api/apiclient/README.md), or you just want to control what's going on at a lower level, read on.

First, refresh the list of available updates:
```shell
apiclient raw -u /actions/refresh-updates -m POST
```

Now you can see the list of available updates, along with the chosen update, according to your `version-lock` [setting](../../README.md#updates-settings):
```shell
apiclient get /updates/status
```

This will return the current update status in JSON format. The status should look something like the following (pretty-printed):
```json
{
  "update_state": "Available",
  "available_updates": [
    "0.4.0",
    "0.3.4",
    ...
  ],
  "chosen_update": {
    "arch": "x86_64",
    "version": "0.4.0",
    "variant": "aws-k8s-1.15"
  },
  "active_partition": {
    "image": {
      "arch": "x86_64",
      "version": "0.3.2",
      "variant": "aws-k8s-1.15"
    },
    "next_to_boot": true
  },
  "staging_partition": null,
  "most_recent_command": {
    "cmd_type": "refresh",
    "cmd_status": "Success",
    ...
  }
}
```

You can see that we're running `v0.3.2` in the active partition, and that `v0.4.0` is available.
If you're happy with that selection, you can request that the update be downloaded and applied to disk.
(The update will remain inactive until you make the `activate-update` call below.)
```shell
apiclient raw -u /actions/prepare-update -m POST
```

After you request that the update be prepared, you can check the update status again until it reflects the new version in the staging partition.
```shell
apiclient get /updates/status
```

If the staging partition shows the new version, you can proceed to "activate" the update.
This means that as soon as the host is rebooted it will try to run the new version.
(If the new version can't boot, we automatically flip back to the old version.)
```shell
apiclient raw -u /actions/activate-update -m POST
```

You can reboot the host with:
```shell
apiclient raw -u /actions/reboot -m POST
```
