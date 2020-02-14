#!/usr/bin/env bash

# Spins up an EKS test cluster with no initial worker nodes using 'eksctl'. Outputs an env file containing information
# used for setting up cluster worker nodes for Kubernetes conformance testing.

# If the script execution is interrupted or terminated halfway, it automatically tries to clean up allocated resources.

# Process flow:
# * Use 'eksctl' to set up cluster with no initial nodes and outputs the kubeconfig file in the current directory with name 'cluster-name-config'
# * Modify security groups to allow traffic needed for Kubernetes conformance testing.
# * Generate a 'cluster-name.env' file containing cluster information in current directory

# Environment assumptions:
# * aws-cli is set up (via environment or config) to operate EC2 in the given region.
# * Some required tools are available locally; look just below these comments.
# * AWS account has space for an additional VPC (maximum is 5) in specified region.

# Caveats:
# * Certain us-east-1 AZs (e.g. us-east-1e) do not support Amazon EKS. If that happens, 'eksctl' will prompt an error.
#   See: https://github.com/weaveworks/eksctl/issues/817. Use '--zones' to specify AZ to ensure that doesn't happen

# Check for required tools
for tool in jq aws kubectl eksctl; do
  if ! command -v ${tool} > /dev/null; then
    echo "* Can't find executable '${tool}'" >&2
    exit 2
  fi
done

DEFAULT_CLUSTER_NAME=sonobuoy-test
CNI_PLUGIN_CONFIG=https://raw.githubusercontent.com/aws/amazon-vpc-cni-k8s/release-1.6/config/v1.6/aws-k8s-cni.yaml

# Helper functions

usage() {
  cat >&2 <<EOF
${0##*/}
                 --region <region>
                 [ --zones us-west-2a,us-west-2b ]
                 [ --cluster-name my-test-cluster ]
Spins up EKS test cluster with no initial worker nodes with 'eksctl' and outputs an env file containing information used
for setting up cluster worker nodes for Kubernetes conformance testing.

Required:
   --region                     The AWS region

Optional:
   --zones                      The availablility zones. Two required if specified. (e.g us-west-2a,us-west-2b)
   --cluster-name               Name of the cluster to create with 'eksctl'. (default ${DEFAULT_CLUSTER_NAME})
EOF
}

required_arg() {
  local arg="${1:?}"
  local value="${2}"
  if [ -z "${value}" ]; then
    echo "ERROR: ${arg} is required" >&2
    exit 2
  fi
}

parse_args() {
  while [ ${#} -gt 0 ] ; do
    case "${1}" in
      --region ) shift; REGION="${1}" ;;

      --cluster-name ) shift; CLUSTER_NAME="${1}" ;;
      --zones ) shift; ZONES="${1}" ;;

      --help ) usage; exit 0 ;;
      *)
        echo "ERROR: Unknown argument: ${1}" >&2
        usage
        exit 2
        ;;
    esac
    shift
  done

  CLUSTER_NAME="${CLUSTER_NAME:-${DEFAULT_CLUSTER_NAME}}"

  # Required arguments
  required_arg "--region" "${REGION}"
}

cleanup() {
  if [ -n "${eks_cluster_creation_attempted}" ]; then
    echo "Deleting the test cluster, whole process can take up to 15 minutes"
    eksctl delete cluster -r "${REGION}" -n "${CLUSTER_NAME}" -w
    exit_on_error ${?} "* Failed to delete ${CLUSTER_NAME} with eksctl; there might be leftover CloudFormation stacks that needs to be deleted. Look for eksctl-${CLUSTER_NAME}-*" >&2
  fi
}

trap 'cleanup' EXIT SIGINT SIGTERM

exit_on_error() {
  local rc="${1:?}"
  local msg="${2:?}"

  if [ "${rc}" -ne 0 ]; then
    echo "${msg}" >&2
    exit 1
  fi
}

# Initial setup and checks
parse_args "${@}"

echo "Setting up fresh EKS cluster with eksctl"
eksctl get cluster -r "${REGION}" -n "${CLUSTER_NAME}" > /dev/null 2>&1
if [ "${?}" -eq 0 ]; then
  echo "* An EKS cluster already exists with name ${CLUSTER_NAME}" >&2
  exit 1
fi

eks_cluster_creation_attempted=true
eksctl create cluster -r "${REGION}" --zones "${ZONES}" -n "${CLUSTER_NAME}" --nodes 0
exit_on_error ${?} "* Failed to set up EKS cluster with eksctl"

kubeconfig_file="${CLUSTER_NAME}"-config
echo "Writing kubeconfig for ${CLUSTER_NAME} to ${kubeconfig_file}"
eksctl utils write-kubeconfig -r "${REGION}" -c "${CLUSTER_NAME}" --kubeconfig "${kubeconfig_file}"
exit_on_error ${?} "* Failed to write kube config"

KUBECTL="kubectl --kubeconfig ${kubeconfig_file}"
echo "Apply configuration for AWS CNI plugin"
${KUBECTL} apply -f "${CNI_PLUGIN_CONFIG}"
exit_on_error ${?} "* Failed to apply configuration for AWS CNI plugin"

echo "Generating userdata file for launching Bottlerocket worker nodes"
endpoint=$(set -o pipefail; \
  eksctl get cluster -r "${REGION}" -n "${CLUSTER_NAME}" -o json \
  | jq --raw-output '.[].Endpoint')
exit_on_error ${?} "* Failed to get cluster endpoint"

certificate_authority=$(set -o pipefail; \
  eksctl get cluster -r "${REGION}" -n "${CLUSTER_NAME}" -o json \
  | jq --raw-output '.[].CertificateAuthority.Data')
exit_on_error ${?} "* Failed to get cluster certificate authority"

userdata_file="${CLUSTER_NAME}"-user-data.toml
cat > "${userdata_file}" <<EOF
[settings.kubernetes]
api-server = "${endpoint}"
cluster-name = "${CLUSTER_NAME}"
cluster-certificate = "${certificate_authority}"
EOF
exit_on_error ${?} "* Failed to write userdata file"

echo "Getting instance profile for use in instance launches"
INSTANCE_PROFILE=$(set -o pipefail; \
  aws iam list-instance-profiles --output json \
  | jq --raw-output ".InstanceProfiles[].InstanceProfileName | select(match(\"eksctl-${CLUSTER_NAME}-.*-NodeInstanceProfile-\"))")

if [ -z "${INSTANCE_PROFILE}" ]; then
  echo "* Failed to get instance profile" >&2
  exit 1
fi

echo "Setting up security groups"
eks_subnet_ids="$(set -o pipefail; \
  eksctl get cluster -r "${REGION}" -n "${CLUSTER_NAME}" -o json \
  | jq --raw-output '.[].ResourcesVpcConfig.SubnetIds[]')"
exit_on_error ${?} "* Failed to get subnet IDs of the EKS cluster"

subnet_ids=($(set -o pipefail; \
  aws ec2 describe-subnets --subnet-ids \
  ${eks_subnet_ids[@]} \
  --region "${REGION}" \
  --filters "Name=tag:Name,Values=eksctl-${CLUSTER_NAME}-cluster/SubnetPrivate*" \
  --output json | jq --raw-output '.Subnets[].SubnetId'))
exit_on_error ${?} "* Failed to get subnet ID for launching bottlerocket worker nodes"


# Allow TCP traffic over ports 1-1024 for Kubernetes conformance testing
nodegroup_sg=$(set -o pipefail; \
  aws ec2 describe-security-groups \
  --region "${REGION}" \
  --filters "Name=tag:Name,Values=*${CLUSTER_NAME}-nodegroup*" \
  --query "SecurityGroups[*].{Name:GroupName,ID:GroupId}" \
  --output json | jq --raw-output '.[].ID')
exit_on_error ${?} "* Failed to get nodegroup security group ID"

clustershared_sg=$(set -o pipefail; \
  aws ec2 describe-security-groups \
  --region "${REGION}" \
  --filters "Name=tag:Name,Values=*${CLUSTER_NAME}*ClusterShared*" \
  --query "SecurityGroups[*].{Name:GroupName,ID:GroupId}" \
  --output json | jq --raw-output '.[].ID')
exit_on_error ${?} "* Failed to get cluster shared security group ID"

controlplane_sg=$(set -o pipefail; \
  aws ec2 describe-security-groups \
  --region "${REGION}" \
  --filters "Name=tag:Name,Values=*${CLUSTER_NAME}*ControlPlane*" \
  --query "SecurityGroups[*].{Name:GroupName,ID:GroupId}" \
  --output json | jq --raw-output '.[].ID')
exit_on_error ${?} "* Failed to get control plane security group ID"

aws ec2 authorize-security-group-ingress \
  --region "${REGION}" \
  --group-id "${nodegroup_sg}" \
  --protocol tcp \
  --port 1-1024 \
  --source-group "${controlplane_sg}"
exit_on_error ${?} "* Failed to authorize nodegroup sg ingress rules"

aws ec2 authorize-security-group-egress \
  --region "${REGION}" \
  --group-id "${controlplane_sg}" \
  --protocol tcp \
  --port 1-1024 \
  --source-group "${nodegroup_sg}"
exit_on_error ${?} "* Failed to authorize control plane sg egress rules"

echo "Generating env file for launching Bottlerocket worker nodes"
cat > "${CLUSTER_NAME}.env" <<EOF
CLUSTER_NAME="${CLUSTER_NAME}"
REGION="${REGION}"
SUBNET_ID="${subnet_ids[0]}"
NODEGROUP_SG="${nodegroup_sg}"
CONTROLPLANE_SG="${controlplane_sg}"
CLUSTERSHARED_SG="${clustershared_sg}"
INSTANCE_PROFILE="${INSTANCE_PROFILE}"
KUBECONFIG_FILE="${kubeconfig_file}"
USERDATA_FILE="${userdata_file}"
EOF
echo "Finished setting up test EKS cluster."
echo "Userdata file: ${CLUSTER_NAME}-user-data.toml"
echo "Env file: ${CLUSTER_NAME}.env"
unset eks_cluster_creation_attempted
