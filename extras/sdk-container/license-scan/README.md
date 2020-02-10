# Bottlerocket's license-scan tool

In a traditional package-based Linux distribution, upstream sources installed on the host have entries in a database that list the name, upstream URL, and license information.
The license files are generally also provided on the filesystem.

Bottlerocket is not a package-based Linux distribution, so an alternative to attributing third-party software must be present.
The **license-scan** tool scans third-party code from sources such as the Go vendor directory or the Cargo dependency graph.
It writes attribution information for the software, along with the result of a license scan and copies of the license files, to a directory structure.

The Bottlerocket build system uses this tool to generate the /usr/share/licenses directory for Go and Rust projects.

## Logic

license-scan determines the individual projects to scan, then walks the project structures for [files that are named like license files according to the ignore crate](https://github.com/BurntSushi/ripgrep/blob/ignore-0.4.9/ignore/src/types.rs#L173-L199).

Those files are scanned by [askalono](https://github.com/amzn/askalono), at a 93% confidence interval, to determine license matches.
Users can override the scanned licenses of a particular package with a clarification file (see below).

An attribution.txt file and copies of all the license files are written to the output directory.
The attribution.txt contains the name of the package, the URL if it's reasonable to determine, and the detected `SPDX-License-Identifier` expression.

Some types of files, like `NOTICE` or `PATENTS`, are statements to distribute along with the license text, but are not themselves scannable licenses.
If they fail to scan, they're ignored but still copied into the output directory.

## Use in the Bottlerocket build system

The RPM macro `%cross_scan_attribution` calls `bottlerocket-license-scan` with the `--spdx-data` and `--out-dir` options.
The package's spec file can list additional options, then lists the subcommand and any subcommand options.
For example, a Go project might write:

```plain
%cross_scan_attribution go-vendor vendor
```

to scan a Go vendor directory at `vendor` and place the license data in `%{buildroot}/usr/share/licenses`.

## Clarification file example

```toml
# The quoted portion is the go path
[clarify."sigs.k8s.io/yaml"]
# the SPDX-License-Identifier expression the software is licensed under
expression = "MIT AND BSD-3-Clause"
# license files the expression applies to
license-files = [
    { path = "LICENSE", hash = 0xcdf3ae00 },
]

# and you can have multiple clarification blocks
[clarify."some.example/code"]
...
```
