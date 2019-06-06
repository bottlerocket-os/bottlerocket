{ file ? ./config.toml, config ? null, ... }:
if builtins.isAttrs config then
  config
else
  builtins.fromTOML (builtins.readFile file)


