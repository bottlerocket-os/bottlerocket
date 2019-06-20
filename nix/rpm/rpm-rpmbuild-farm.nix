{ stdenvNoCC, lib }:
{ name, sources, specs, rpms }:
let
  linker = dest: entries:
    lib.concatMapStrings (s: "ln -s '${s.path}/*' '${dest}/${s.name}';\n") entries;
  
  linkChildren = dest: paths:
    linker dest (map (p: { name = ""; path = p; }) paths);
  linkDrvs = dest: drvs:
    linker dest (map (d: { name = d.name; path = d.out; }) drvs);
in
stdenvNoCC.mkDerivation {
  name = "${name}-rpmbuild-farm";
  
  phases = [ "setupPhase" "sourcesInstallPhase" "specsInstallPhase" "rpmsInstallPhase" ];
  
  setupPhase = "mkdir -p $out/{SOURCES,SPECS,RPMS}";
  sourcesInstallPhase = linker "SOURCES" sources;
  specsInstallPhase = linker "SPECS" specs;
  rpmsInstallPhase = linker "RPMS" rpms;
}
