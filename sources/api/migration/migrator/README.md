# migrator

Current version: 0.1.0

migrator is a tool to run migrations built with the migration-helpers library.

It must be given:
* a data store to migrate
* a version to migrate it to
* where to find migration binaries

Given those, it will:
* confirm that the given data store has the appropriate versioned symlink structure
* find the version of the given data store
* find migrations between the two versions
* if there are migrations:
  * run the migrations; the transformed data becomes the new data store
* if there are *no* migrations:
  * just symlink to the old data store
* do symlink flips so the new version takes the place of the original

To understand motivation and more about the overall process, look at the migration system
documentation, one level up.

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
