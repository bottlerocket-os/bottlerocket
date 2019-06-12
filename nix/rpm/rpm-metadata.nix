{ stdenv, lib, config, rpm, rpm-macros, mkMacroPath, ... }:
let
  cfg = config.builder.rpm;
  image = cfg.container-image;

  # TODO: use cross arch here
  # Architecture specific macros
  arch-macros = "${rpm-macros.arch}/x86_64";
  # The base set of thar macros
  thar-macros = "${rpm-macros.out}/*";
  # RPM distributed macros
  rpm-macros = "${rpm}/lib/rpm/macros";
  
  macroPath = mkMacroPath [ arch-macros thar-macros rpm-macros ];
in
{ rpm-spec, rpm-sources, ... }:
stdenv.mkDerivation rec {
  name = "rpm-metadata-${baseNameOf rpm-spec}";
  src = rpm-spec;
  buildInputs = [ rpm ];
  phases = ["parsePhase" "generatePhase"];

  # Parse the rpm spec to extract metadata.
  parsePhase = ''
  mkdir -p $out
  rpmspec "--macros=${macroPath}" --define "_sourcedir ./" --parse ${rpm-spec} > $out/parsed.spec
  if grep -o -E '^Source[0-9]+:.*http.*$' $out/parsed.spec | sed 's/Source.*:.*http/http/' | grep -v -e '^$' -e '.crate$' | tee remote-source-urls; then
    echo "Collecting sources for package"
  else
    echo "Package has no sources"
  fi
  '';
  
  generatePhase = ''
  set -x
  declare -A source_hash_entry
  while read SOURCE_URL; do
    echo "Generating source entry for $SOURCE_URL"
    FILENAME="''${SOURCE_URL##*/}"
    # ALGO-HASH_CONTENT - https://www.w3.org/TR/SRI/ 
    SRI="$(awk -v filename="($FILENAME)" '$2 == filename {print tolower($1)"-"$4}' ${rpm-sources})"
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
