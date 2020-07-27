# ecs-agent

The ecs-agent package in Bottlerocket provides the ECS agent and a systemd unit
that sets up necessary configuration on the host.

This README is temporary and is meant to track the known issues and remaining
work items for the ECS agent.

## Known issues

* The `docker` CLI is included in the variant.  We should determine whether we
  want to keep it or not; it's useful for debugging but it's not a
  strictly-necessary component.
* CNI plugins are not yet packaged - This means that awsvpc mode and AppMesh
  are both currently unsupported.
  * The log path of CNI plugins - currently all of them are defaulting to the
    container path of the log directory bind mount, e.g.
    https://github.com/aws/amazon-ecs-cni-plugins/blob/0c6216c60401232805e50e31d4040ae84b0b23cf/plugins/eni/main.go#L32
    https://github.com/aws/amazon-ecs-agent/blob/master/agent/ecscni/plugin.go?rgh-link-date=2020-07-25T00%3A20%3A16Z#L39
  * The path to the process's network namespace handle is currently hardcoded
    to the container path of the procfs bind mount
    https://github.com/aws/amazon-ecs-agent/blob/782948476da6d995ad33c6a04130f8172820af27/agent/ecscni/types.go#L38
* Logging is currently set to `debug` to assist with development.
* The systemd unit contains many `ExecStartPre`/`ExecStopPost` commands, with
  little explanation or infrastructure.  The `ExecStartPre` commands should
  probably be run exactly once, and the `ExecStopPost` commands probably
  shouldn't ever run.
* The Bottlerocket datastore does not accept keys with `\` characters.  This
  means that even though `\` is valid in ECS attribute names, it is not
  supported on Bottlerocket.
