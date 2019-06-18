{ writeScript, docker-cli }:
writeScript "docker-sanity-check" ''
if ! test -e /var/run/docker; then
  echo "docker socket is missing!"
  echo "add '--option extra-sandbox-paths /var/run/docker.sock' to your args"
  exit 1
fi
''
