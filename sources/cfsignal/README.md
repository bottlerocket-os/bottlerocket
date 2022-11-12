# cfsignal

Current version: 0.1.0

### Introduction

Cfsignal is similar to [cfn-signal](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/cfn-signal.html).

When creating an Auto Scaling Group, CloudFormation can be configured to wait for the expected number of signals from instances before considering the ASG successfully created. See [CreationPolicy](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-attribute-creationpolicy.html) and [UpdatePolicy](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-attribute-updatepolicy.html) for more details.

Cfsignal uses `systemctl is-system-running` to determine whether the boot has succeeded or failed, and sends the corresponding signal to the CloudFormation stack.

### Configuration

Configuration is read from a TOML file, which is generated from Bottlerocket settings:
* `should_signal`: Whether to check system status and send signal.
* `stack_name`: Name of the CFN stack to signal.
* `logical_resource_id`: The logical ID of the AutoScalingGroup resource that you want to signal.

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
