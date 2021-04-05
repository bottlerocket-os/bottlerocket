# ecs-agent

The ecs-agent package in Bottlerocket provides the ECS agent and a systemd unit
that sets up necessary configuration on the host.

This README is temporary and is meant to track the known issues and remaining
work items for the ECS agent.

## Known issues

* The `docker` CLI is included in the variant.  We should determine whether we
  want to keep it or not; it's useful for debugging but it's not a
  strictly-necessary component.
* The systemd unit contains many `ExecStartPre`/`ExecStopPost` commands, with
  little explanation or infrastructure.  The `ExecStartPre` commands should
  probably be run exactly once, and the `ExecStopPost` commands probably
  shouldn't ever run.
* The Bottlerocket datastore does not accept keys with `\` characters.  This
  means that even though `\` is valid in ECS attribute names, it is not
  supported on Bottlerocket.
