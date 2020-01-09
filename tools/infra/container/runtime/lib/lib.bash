# shellcheck shell=bash

# Logger provides the corrected interface to log to stderr.
logger() {
    command logger --no-act -s "$@"
}
