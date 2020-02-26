#!/usr/bin/env bash
DOCS=(START.md README.md BUILDING.md QUICKSTART.md CHANGELOG.md extras/dogswatch/README.md)
EXTRAS=(extras/dogswatch/{dogswatch,dev/deployment}.yaml)

if ! hash grip; then
    >&2 echo "grip is not installed, run 'pip3 install --user grip'"
    exit 1
fi

top=$(git rev-parse --show-toplevel)
mkdir -p "${top}/html"
for doc in "${DOCS[@]}"; do
    out="${top}/html/${doc%.md}.html"
    mkdir -p "$(dirname "$out")"
    grip --title="${doc}" --export \
        <(
            cat <<'EOF'
@@BOTTLEROCKET-SENTINEL-START@@
**The best way to get in touch with the Bottlerocket development team** during our preview
is via [thar-preview@amazon.com](mailto:thar-preview@amazon.com)
or #thar-preview on the [awsdevelopers Slack workspace](https://awsdevelopers.slack.com) (email us for an invite).
We'd love to talk with you and hear your feedback on Bottlerocket!
<br><br>
[&larr; Documentation index](/START.md)

---

EOF
            cat "${top}/${doc}"
        ) \
        "${out}"
    sed -i \
        -e 's/.*<link rel="stylesheet".*octicons\.css.*/<style>.markdown-body .anchor span:before { font-size: 16px; content: "\\1f517"; }<\/style>/' \
        -e '/<link rel="icon"/d' \
        -e 's/<p>@@BOTTLEROCKET-SENTINEL-START@@/<p style="background-color: #a8dfee; border: 1px solid #008296; padding: 1em;">/' \
        -e 's/<a href="\([^ ">]*\).md/<a href="\1.html/g' \
        -e 's^<a href="\.\./\.\./pull/[^ ">]*">\(#[0-9]\+\)</a>^\1^g' \
        "${out}"
done

for extra in "${EXTRAS[@]}"; do
    out="${top}/html/${extra}"
    echo "Copying ${extra} to ${out}"
    install -D "${extra}" "${out}"
done
