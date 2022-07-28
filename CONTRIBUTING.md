# Contributing Guidelines

Thank you for your interest in contributing to our project.
Whether it's a bug report, new feature, correction, or additional documentation, we greatly value feedback and contributions from our community.

Please read through this document before submitting any issues or pull requests to ensure we have all the necessary information to effectively respond to your bug report or contribution.


## Reporting Bugs/Feature Requests

We welcome you to use the GitHub issue tracker to report bugs or suggest features.

When filing an issue, please check [existing open](https://github.com/bottlerocket-os/bottlerocket/issues) and [closed](https://github.com/bottlerocket-os/bottlerocket/issues?q=is%3Aissue+is%3Aclosed) issues to make sure somebody else hasn't already reported the issue.
Please try to include as much information as you can.
Details like these are incredibly useful:

* A reproducible test case or series of steps
* The version of our code being used
* Any modifications you've made relevant to the bug
* Anything unusual about your environment or deployment


## Contributing via Pull Requests
Contributions via pull requests are much appreciated.
Before starting a pull request, please ensure that:

1. You open an issue first to discuss any significant work - we would hate for your time to be wasted.
2. You are working against the latest source on the *develop* branch.
3. You check existing [open](https://github.com/bottlerocket-os/bottlerocket/pulls) and [merged](https://github.com/bottlerocket-os/bottlerocket/pulls?q=is%3Apr+is%3Aclosed) pull requests to make sure someone else hasn't addressed the problem already.

To send us a pull request, please:

1. Fork the repository.
2. Modify the source; please focus on the specific change you are contributing. If you also reformat the code, it will be hard for us to focus on your change.
3. Ensure local tests pass.
4. Commit to your fork using clear commit messages.
5. Send us a pull request, answering any default questions in the pull request interface.
6. Pay attention to any automated CI failures reported in the pull request, and stay involved in the conversation.

GitHub provides additional documentation on [forking a repository](https://help.github.com/articles/fork-a-repo/) and [creating a pull request](https://help.github.com/articles/creating-a-pull-request/).

## Repo branch and tag structure 

Active development occurs under the `develop` branch.

Bottlerocket uses both tags and branches for release alignment. Numbered releases are always associated with [tags that mirror the full SemVer 3-digit version number](https://github.com/bottlerocket-os/bottlerocket/tags) (e.g. `1.7.2`). [Branches are for patching only](https://github.com/bottlerocket-os/bottlerocket/branches/all): if a patch is required, a branch will be cut for that minor release line (e.g. `1.7.x`). As a consequence, some previous minor versions may not have a branch if they never required a subsequent patch.

## Filename case conventions

Bottlerocket follows a few basic filename case conventions:

- All extensions are lowercase,
- Build related configuration files always start with a capital letter (e.g. `Infra.toml`, `Release.toml`),
- All caps is used for documents and licenses (e.g. `PUBLISHING.md`, `TRADEMARKS.md`),
- All lower case is used for all other files (e.g. `sample-eksctl.yaml`, `main.rs`).


## Finding contributions to work on
Looking at the existing issues is a great way to find something to contribute on.
As this repository uses GitHub issue [labels](https://github.com/bottlerocket-os/bottlerocket/labels), looking at any ['status/helpwelcome'](https://github.com/bottlerocket-os/bottlerocket/labels/status%2Fhelpwelcome) issues is a great place to start.


## Code of Conduct
This project has adopted the [Amazon Open Source Code of Conduct](https://aws.github.io/code-of-conduct).
For more information see the [Code of Conduct FAQ](https://aws.github.io/code-of-conduct-faq) or contact opensource-codeofconduct@amazon.com with any additional questions or comments.


## Security issue notifications
If you discover a potential security issue in this project we ask that you notify AWS/Amazon Security via our [vulnerability reporting page](http://aws.amazon.com/security/vulnerability-reporting/).
Please do **not** create a public GitHub issue.


## Licensing

See the [COPYRIGHT](COPYRIGHT) file for our project's licensing.
We will ask you to confirm the licensing of your contribution.
