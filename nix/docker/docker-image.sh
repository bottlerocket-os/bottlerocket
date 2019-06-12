dockerImageLabeled() { 
    local label="${1:-containerImage}"
    local value="${2:-${!label}}"
    docker images --filter "label=$label=$value" --format "{{.ID}}" --no-trunc
}

dockerDaemonRunning() {
    docker info >/dev/null || \
	echo "ERROR: docker daemon is not running or reachable" && exit 1
}

dockerSaveToOutput() {
    local imageID="${1:-$(dockerImageLabeled)}"
    local imageTarget="${2:-$containerImage}"
    docker save "$imageID" | $gzip -c - > "$imageTarget"
    echo "$imageID" > "${containerImageID:-/dev/null}"
}
