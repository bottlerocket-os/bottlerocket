# Simplistic one pass dependency solver for pulling in RPMs
# automatically.
{ lib, tharPackages }:

{ requires, packageSet ? tharPackages }:
with lib;
assert assertMsg (isList requires)
  "require must be a list of requirements to satisfy";
let
  flatMap = f: l: flatten (map f l);
  
  hasReq = req: provide: let
    reqNamePrefix = "${req} =";
  in
    hasPrefix reqNamePrefix provide;
  # Find attributes that satisfies the requirements in the list.
  satisfyReq = attrs: req:
    let
      candidates = filterAttrs (n: v: v ? rpmMetadata) attrs;
      matcher = n: v: any (hasReq req) v.rpmMetadata.provides;
      matched = filterAttrs matcher candidates;
    in
      matched;
  satisfyReqs = reqs: attrs:
    let
      res = flatMap (satisfyReq attrs) (flatten reqs);
    in
      foldl (a: x: a // x) {} res;
  
  directDeps = satisfyReqs requires packageSet;
  indirectDeps =
    let
      indirectRequires = mapAttrsToList (n: p: p.rpmMetadata.requires) directDeps;
    in
      satisfyReqs indirectRequires packageSet;
  deps = let
    byName = (a: b: a.name > b.name);
    combined = flatMap attrValues [ directDeps indirectDeps ];
  in sort byName combined;
in
unique deps

