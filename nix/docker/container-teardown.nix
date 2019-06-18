{ writeScript }:
writeScript "docker-container-teardown" ''
test -e $out && chown -hR "$euid:$egid" $out
''
