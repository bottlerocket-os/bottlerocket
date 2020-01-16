# shellcheck shell=bash

# Logger provides the corrected interface to log to stderr.
logger() {
    # Use logger if its usable
    if test -S /dev/log; then
        command logger --no-act -s "$@"
        return 0
    fi

    # Otherwise, use a simple polyfill implementation that provides a similar
    # enough interface to be used across scripts.
    local tag
    local message
    local format

    # polyfill in a logger that writes to stderr
    while [ "$#" -ne 0 ]; do
        case "$1" in
            -t )
                tag="$2"
                shift 1
                ;;
            -*|-- )
                # drop options
                ;;
            * )
                # message
                message=( "$@" )
                break
                ;;
        esac
        shift 1
    done

    # Message printer format
    format="${tag:+"$tag: "}%s\n"

    # Single message in function call
    if [[ "${#message[@]}" -ne 0 ]]; then
        printf "$format" "${message[*]}" >&2
        return 0
    fi

    # Stream of messages sent to function as input
    while read msg; do
        printf "$format" "${msg}" >&2
    done

    return 0
}

# has_command returns true for present commands
has_command() {
    local name="${1:?has_command requires a name to check}"
    command -v "$name" &>/dev/null
}
