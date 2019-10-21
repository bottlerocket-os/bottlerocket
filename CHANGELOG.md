# v0.1.6 (2019-10-21)

## OS changes

* The system fetches the pause container from ECR before starting `kubelet` (#382).
* New settings: `settings.kubernetes.node-labels` and `settings.kubernetes.node-taints` (#390, #408).
* The control container has an `enable-admin-container` helper (#405, #413).
* Rust dependencies updated (#410).
* `thar-be-settings` added trace-level messages in the client module (#411).
* `updog` no longer checks for migrations from new root images (#416).
* `pluto` was cleaned up to create an HTTP connection more consistently (#419).
* Settings that are usually generated may have defaults, and `settings.kubernetes.max-pods` defaults to `110` if the EC2 instance type cannot be determined (#420).
* The admin container MOTD is clearer about where the host's filesystem is mounted (#424).
* `block-party` (used in `growpart` and `signpost`) errors are better structured (#425).
* `thar-be-settings` logs render errors when running in `--all` mode (#427).
* [Recommended `sysctl` settings from the Kernel Self Protection Project](https://kernsec.org/wiki/index.php/Kernel_Self_Protection_Project/Recommended_Settings#sysctls) are now used (#435).
* `acpid` is enabled by default to handle power button signals sent by EC2 on stop/restart/terminate events (#437).
* `host-ctr` correctly fetches images from non-ECR registries (#439; this regression occurred after v0.1.5).

## Build changes

* amiize uses a short connection timeout when testing SSH connectivity (#409).
* `tuftool` only downloads an arbitrary `root.json` with `--allow-root-download` (#421).
* BuildKit updated to v0.6.2 (#423, #429).
* First-party Rust code is built in the same `rpmbuild` invocation to improve build times (#428).
* `tuftool` accepts relative dates (#438).

## Documentation changes

* The `workspaces/updater` crates are better documented (#381).
* INSTALL.md's subnet selection documentation is improved (#422).
