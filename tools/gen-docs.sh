#!/bin/bash
if ! hash grip; then
    >&2 echo "grip is not installed, run 'pip3 install --user grip'"
    exit 1
fi

top=$(git rev-parse --show-toplevel)
mkdir -p "${top}/html"
for doc in START.md README.md INSTALL.md CHANGELOG.md extras/dogswatch/README.md; do
    out="${top}/html/${doc%.md}.html"
    mkdir -p "$(dirname "$out")"
    grip --title="${doc}" --export \
        <(
            cat <<'EOF'
@@THAR-SENTINEL-START@@
**The best way to get in touch with the Thar development team** during this early preview
is via [thar-preview@amazon.com](mailto:thar-preview@amazon.com)
or #thar-preview on the [awsdevelopers Slack workspace](https://awsdevelopers.slack.com) (email us for an invite).
We'd love to talk with you and hear your feedback on Thar!
<br><br>
[&larr; Documentation index](/START.md)

---

EOF
            cat "${top}/${doc}"
        ) \
        "${out}"
    sed -i \
        -e '/<link rel="stylesheet".*octicons\.css/d' \
        -e '/<link rel="icon"/d' \
        -e 's/<p>@@THAR-SENTINEL-START@@/<p style="background-color: #a8dfee; border: 1px solid #008296; padding: 1em;">/' \
        -e 's/<a href="\(.*\).md\(#.*\)\?">/<a href="\1.html\2">/g' \
        -e 's^<a href="\.\./\.\./pull/.*">\(#[0-9]\+\)</a>^\1^g' \
        "${out}"
done

for extra in extras/dogswatch/{dogswatch,dev/deployment}.yaml; do
    out="${top}/html/${extra}"
    echo "Copying to ${out}"
    mkdir -p "$(dirname "$out")"
    cp "${extra}" "${out}"
done
