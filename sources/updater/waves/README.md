# Update waves

As mentioned in the [Bottlerocket update overview](../README.md), OS updates can include "waves" for staggered deployment.
These waves are defined in the `manifest.json`, which lives in the TUF repository.
Each time an OS update is made available, `manifest.json` is updated with the information pertinent to that update using [updata](../updog) or its related libraries.
Waves may be supplied to `updata` on the command line, passed as a TOML-formatted file.

This directory contains a few examples of these update wave files.
Specific details are encapsulated in each file, but they are:

* `default-waves.toml`: A "normal" deployment
* `accelerated-waves.toml`: An accelerated deployment
* `ohno.toml`: An extremely accelerated deployment in case of emergency.

## Understanding the concept of waves

Waves include a *seed* and a *start time*.

Each Bottlerocket node generates a "seed" for itself which is simply a number between 0-2048 that determines where it falls in the update order.
Nodes that have a seed within the current wave will update.
All waves include the seeds of the prior wave, so if a node misses its wave for whatever reason, it still updates at a later time.

## Writing wave files

Wave files must be [valid TOML](https://github.com/toml-lang/toml) containing a list of `[[waves]]` entries.
Waves defined in these files must contain two keys, `start_after` and `fleet_percentage`.

`start_after` must be:

* a valid RFC3339 formatted string OR
* a string like `"7 days"` or `"2 hours"`. Additional details about valid strings can be found [here](../../parse-datetime)

It represents an offset of time starting from when the operator updates the `manifest.json` file, NOT an offset starting at the time `manifest.json` is uploaded to S3. In simple terms, it is "now" plus whatever time period is specified.

`fleet_percentage` must be an unsigned integer from 1 to 100.
It represents the desired total percentage of the fleet to be updated by the time this wave is over.
This percentage maps directly to the seed value; it's the percentage of the maximum seed, 2048.

Please see the files in this directory for proper examples.
