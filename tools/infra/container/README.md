# Container Environments

Container images, defined in this directory, provide environments for infra's build and automation needs.

## Images

Each image is defined in their own `Dockerfile` and suffixed with its name. For example the `builder` container - used in CI builds - is defined by `Dockerfile.builder`. 
The containers copy in common resources and others as needed from this shared root context.

**`builder` image**

The `builder` image provides an environment in which packages and images may be built.
`builder`'s container image is created with all required dependencies used by the build driver, `buildsys`, and the supporting tools & scripts used by it (including many of the `cargo-make` tasks' dependencies).

# Building

## Development Images

To all build images locally, a single `make` call can be made:

```bash
make all
```

Each `Dockerfile.<name>` can be built individually with `make $name` as needed.

## Release Images

As with the development images, all images may be built at once:

```bash
make release
```

To build a specific image, for instance named `builder`, `make` may be provided this name to build its release image:

```bash
make release NAME=builder
```
