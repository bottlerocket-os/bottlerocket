#!/bin/bash
if ! hash grip; then
    >&2 echo "grip is not installed, run 'pip3 install --user grip'"
    exit 1
fi

top=$(git rev-parse --show-toplevel)
mkdir -p "${top}/html"
for doc in README.md INSTALL.md CHANGELOG.md; do
    out="${top}/html/${doc%.md}.html"
    grip --title="${doc}" --export \
        <(
            cat <<'EOF'
@@THAR-SENTINEL-START@@
**The best way to get in touch with the Thar development team** during this early preview
is via [thar-preview@amazon.com](mailto:thar-preview@amazon.com)
or #thar-preview on the [awsdevelopers Slack workspace](https://awsdevelopers.slack.com) (email us for an invite).
We'd love to talk with you and hear your feedback on Thar!

---

EOF
            cat "${top}/${doc}"
        ) \
        "${out}"
    sed -i \
        -e '/<link rel="stylesheet".*octicons\.css/d' \
        -e '/<link rel="icon"/d' \
        -e 's/<p>@@THAR-SENTINEL-START@@/<p style="background-color: #a8dfee; border: 1px solid #008296; padding: 1em;">/' \
        "${out}"
done
