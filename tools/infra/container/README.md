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

## Release Images (using a tag)

As with the development images, all images may be built at once:

```bash
make all IMAGE_TAG=release
```

To build a specific image, for instance named `builder`, `make` may be provided this name to build its release image:

```bash
make all NAME=builder IMAGE_TAG=release
```

# Releasing

The `push` target is provided to build & push release container images for use, at least in the context of build and release automation.

The default target will prepare to push the images using the environment's AWS profile to confirm that the ECR repositories line up and subsequently pushing with a default of `IMAGE_TAG=staging`.
This invocation **will** push to the ECR repository, but with the image tagged as "staging".
Doing a push this way will stage the layers in the ECR repository so that subsequent pushes update lightweight references only (pushing a tag that refers to the same layers).

``` bash
make push
```

To push a container image tagged as a release image, which is required for the CodeBuild project to use, the `IMAGE_TAG` must be set explicitly to the same tag that's configured to be pulled by projects.
If the release tag is `release`, then the call to `push` these images would be:

``` bash
make push IMAGE_TAG=release
```

The `Makefile` target would then match the images to their respective ECR repositories, as before, and `docker push` to the images' respective repositories.
If the `make push IMAGE_TAG=release` followed an earlier `make push` then this the `make push IMAGE_TAG=release` call will simply update the references in the remote ECR repository to point to the same layers.
