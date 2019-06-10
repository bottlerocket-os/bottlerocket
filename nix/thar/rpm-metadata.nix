{ stdenv, lib, nixpkgs, config, rpm, ... }:
let
  cfg = config.builder.rpm;
  image = cfg.container-image;
  
  thar-rpm-macros = nixpkgs.callPackage ./rpm-macros.nix {};
  
  arch-macros = "${thar-rpm-macros.arch}/x86_64";
  thar-macros = "${thar-rpm-macros.out}/*";
  rpm-macros = "${rpm}/lib/rpm/macros";
  macroPath = builtins.concatStringsSep ":" [ arch-macros thar-macros rpm-macros ];
in
{ specFile, specSources, ... }:
stdenv.mkDerivation rec {
  name = "rpm-metadata-${baseNameOf specFile}";
  src = specFile;
  buildInputs = [ rpm ];
  phases = ["parsePhase" "generatePhase"];

  # TODO: don't reference the source dir to avoid making its changes
  # part of the invalidation of this drv.

  # Parse the rpmspec for further extraction.
  parsePhase = ''
  set -x
  mkdir -p $out
  rpmspec "--macros=${macroPath}" --define "_sourcedir ${specSources}" --parse ${specFile} > $out/parsed.spec
  cat $out/parsed.spec
  if grep -o -E '^Source[0-9]+:.*http.*$' $out/parsed.spec | sed 's/Source.*:.*http/http/' | grep -v -e '^$' -e '.crate$' | tee remote-source-urls; then
    echo "Collecting sources for package"
  else
    echo "Package has no sources"
  fi
  set +x
  '';
  
  generatePhase = ''
  set -x
  declare -A source_hash_entry
  while read SOURCE_URL; do
    echo "Generating source entry for $SOURCE_URL"
    FILENAME="''${SOURCE_URL##*/}"
    # ALGO-HASH_CONTENT - https://www.w3.org/TR/SRI/ 
    SRI="$(awk -v filename="($FILENAME)" '$2 == filename {print tolower($1)"-"$4}' "${specSources}/sources")"
    test -n "$SRI" || exit 1
    source_hash_entry["$SOURCE_URL"]="$SRI"
  done < remote-source-urls
  
  json_entries=""
  for url in "''${!source_hash_entry[@]}"; do
    echo "Adding source entry for $url"
    if [[ -n "$json_entries" ]]; then json_entries="$json_entries, "; fi
    urlHash="''${source_hash_entry[$url]}"
    # SRI prefixed with algo
    urlHashAlgo="''${urlHash%%-*}"
    # Stripped hash
    urlAlgoHash="''${urlHash##*-}"
    entry="$(printf '{"url": "%s", "%s": "%s"}' "$url" "$urlHashAlgo" "$urlAlgoHash")"
    json_entries+="$entry"
  done
  printf '{"sources": [%s]}' "$json_entries" | tee "$out/sources.json"
  set +x
  '';
}
