#!/usr/bin/env bash

# Register partitioned root and data images as an AMI in EC2.
# Only registers with HVM virtualization type and GP2 EBS volume type.

# The general process is as follows:
# * Launch a worker instance with EBS volumes that will fit the images
# * Send the images to the instance
# * Write the images to the volumes
# * Create snapshots of the volumes
# * Register an AMI from the snapshots

# Image assumptions:
# * Your images are partitioned, and the root image has a bootloader set up as required.
# * Your root image supports SR-IOV (e1000) and ENA networking.
# * The images fit within the memory of the --instance-type you select.

# Environment assumptions:
# * aws-cli is set up (via environment or config) to operate EC2 in the given region.
# * The SSH key associated with --ssh-keypair is loaded in ssh-agent.
# * Some required tools are available locally; look just below the constants.
# * The worker AMI has rsync, which we use because it can copy/write sparse files.
# * The --security-group-name you specify (or "default") has TCP port 22 open,
#      and you can access EC2 from your location

# Caveats:
# * We try to clean up the worker instance and volumes, but if we're interrupted
#      in specific ways (see cleanup()) they can leak; be sure to check your
#      account and clean up as necessary.

# Tested with the Amazon Linux AMI as worker AMI.
# Example call:
#    bin/amiize.sh --region us-west-2 \
#       --root-image build/images/x86_64-aws-k8s-1.17/latest/bottlerocket-aws-k8s-1.17-x86_64.img \
#       --data-image build/images/x86_64-aws-k8s-1.17/latest/bottlerocket-aws-k8s-1.17-x86_64-data.img \
#       --worker-ami ami-0f2176987ee50226e --ssh-keypair tjk \
#       --instance-type m3.xlarge --name bottlerocket-20190918-01 --arch x86_64 \
#       --user-data 'I2Nsb3VkLWNvbmZpZwpyZXBvX3VwZ3JhZGU6IG5vbmUK'
# This user data disables updates at boot to minimize startup time of this
# short-lived instance, so make sure to use the latest AMI.

# =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

# Constants

# Where to find the volumes attached to the worker instance.
ROOT_DEVICE="/dev/sdf"
DATA_DEVICE="/dev/sdg"
# Where to store the images on the worker instance.
STORAGE="/dev/shm"
# The device names registered with the AMI.
ROOT_DEVICE_NAME="/dev/xvda"
DATA_DEVICE_NAME="/dev/xvdb"
# The default size for the data volume, unless overridden.
DATA_VOLUME_DEFAULT_SIZE="20"

# Features we assume/enable for the images.
VIRT_TYPE="hvm"
VOLUME_TYPE="gp2"
SRIOV_FLAG="--sriov-net-support simple"
ENA_FLAG="--ena-support"

# The user won't know the server in advance.
SSH_OPTS="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null"

# Maximum number of times we'll try to register the images - lets us retry in
# case of timeouts.
MAX_ATTEMPTS=2

# =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

# Early checks

# Check for required tools
for tool in jq aws du rsync dd ssh; do
   what="$(command -v "${tool}")"
   if [ "${what:0:1}" = "/" ] && [ -x "${what}" ]; then
      : # absolute path we can execute; all good
   elif [ -n "${what}" ]; then
      : # builtin or function we can execute; weird but allow flexibility
   else
      echo "** Can't find executable '${tool}'" >&2
      exit 2
   fi
done


# =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

# Helper functions

usage() {
   cat >&2 <<EOF
$(basename "${0}")
                 --root-image <image_file>
                 --data-image <image_file>
                 --region <region>
                 --worker-ami <AMI ID>
                 --ssh-keypair <KEYPAIR NAME>
                 --instance-type INSTANCE-TYPE
                 --name <DESIRED AMI NAME>
                 --arch <ARCHITECTURE>
                 [ --description "My great AMI" ]
                 [ --subnet-id subnet-abcdef1234 ]
                 [ --user-data base64 ]
                 [ --root-volume-size 1234 ]
                 [ --data-volume-size 5678 ]
                 [ --security-group-name default | --security-group-id sg-abcdef1234 ]
                 [ --write-output-dir output-dir ]

Registers the given images as an AMI in the given EC2 region.

Required:
   --root-image               The image file for the AMI root volume
   --data-image               The image file for the AMI data volume
   --region                   The region to upload to
   --worker-ami               The existing AMI ID to use when creating the new snapshot
   --ssh-keypair              The SSH keypair name that's registered with EC2, to connect to worker instance
   --instance-type            Instance type launched for worker instance
   --name                     The name under which to register the AMI
   --arch                     The machine architecture of the AMI, e.g. x86_64, arm64

Optional:
   --description              The description attached to the registered AMI (defaults to name)
   --subnet-id                Specify a subnet in which to launch the worker instance
                              (required if the given instance type requires VPC and you have no default VPC)
                              (must specify security group by ID and not by name if specifying subnet)
   --user-data                EC2 user data for worker instance, in base64 form with no line wrapping
   --root-volume-size         AMI root volume size in GB (defaults to size of disk image)
   --data-volume-size         AMI data volume size in GB (defaults to size of disk image)
   --security-group-id        The ID of a security group name that allows SSH access from this host
   --security-group-name      The name of a security group name that allows SSH access from this host
                              (defaults to "default" if neither name nor ID are specified)
   --write-output-dir         The directory to write out IDs into attribute named files.
                              (not written out to anywhere other than log otherwise)

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
         --root-image ) shift; ROOT_IMAGE="${1}" ;;
         --data-image ) shift; DATA_IMAGE="${1}" ;;
         --region ) shift; REGION="${1}" ;;
         --worker-ami ) shift; WORKER_AMI="${1}" ;;
         --ssh-keypair ) shift; SSH_KEYPAIR="${1}" ;;
         --instance-type ) shift; INSTANCE_TYPE="${1}" ;;
         --name ) shift; NAME="${1}" ;;
         --arch ) shift; ARCH="${1}" ;;

         --description ) shift; DESCRIPTION="${1}" ;;
         --subnet-id ) shift; SUBNET_ID="${1}" ;;
         --user-data ) shift; USER_DATA="${1}" ;;
         --root-volume-size ) shift; ROOT_VOLUME_SIZE="${1}" ;;
         --data-volume-size ) shift; DATA_VOLUME_SIZE="${1}" ;;
         --security-group-name ) shift; SECURITY_GROUP_NAME="${1}" ;;
         --security-group-id ) shift; SECURITY_GROUP_ID="${1}" ;;
         --write-output-dir ) shift; WRITE_OUTPUT_DIR="${1}" ;;

         --help ) usage; exit 0 ;;
         *)
            echo "ERROR: Unknown argument: ${1}" >&2
            usage
            exit 2
            ;;
      esac
      shift
   done

   # Required arguments
   required_arg "--root-image" "${ROOT_IMAGE}"
   required_arg "--data-image" "${DATA_IMAGE}"
   required_arg "--region" "${REGION}"
   required_arg "--worker-ami" "${WORKER_AMI}"
   required_arg "--ssh-keypair" "${SSH_KEYPAIR}"
   required_arg "--instance-type" "${INSTANCE_TYPE}"
   required_arg "--name" "${NAME}"
   required_arg "--arch" "${ARCH}"

   # Validate and canonicalize architecture identifier.
   case "${ARCH,,}" in
      arm64|aarch64)
         ARCH=arm64
         ;;
      x86_64|amd64)
         ARCH=x86_64
         ;;
      *)
         echo "ERROR: Unsupported EC2 machine architecture: $ARCH" >&2
         usage
         exit 2
   esac

   # Other argument checks
   if [ ! -r "${ROOT_IMAGE}" ] ; then
      echo "ERROR: cannot read ${ROOT_IMAGE}" >&2
      exit 2
   fi

   if [ ! -r "${DATA_IMAGE}" ] ; then
      echo "ERROR: cannot read ${DATA_IMAGE}" >&2
      exit 2
   fi

   if [ -n "${SECURITY_GROUP_NAME}" ] && [ -n "${SECURITY_GROUP_ID}" ]; then
      echo "ERROR: --security-group-name and --security-group-id are incompatible" >&2
      usage
      exit 2
   elif [ -n "${SECURITY_GROUP_NAME}" ] && [ -n "${SUBNET_ID}" ]; then
      echo "ERROR: If specifying --subnet-id, must use --security-group-id instead of --security-group-name" >&2
      usage
      exit 2
   fi

   # Defaults
   if [ -z "${SECURITY_GROUP_NAME}" ] && [ -z "${SECURITY_GROUP_ID}" ]; then
      SECURITY_GROUP_NAME="default"
   fi

   if [ -z "${DESCRIPTION}" ] ; then
      DESCRIPTION="${NAME}"
   fi
   # ROOT_VOLUME_SIZE and DATA_VOLUME_SIZE are defaulted below,
   # after we calculate image size
}

cleanup() {
   if [ -n "${instance}" ]; then
      echo "Cleaning up worker instance"
      aws ec2 terminate-instances \
         --output text \
         --region "${REGION}" \
         --instance-ids "${instance}"
      unset instance
   # Clean up volumes if we have them, but *not* if we have an instance - the
   # volumes would still be attached to the instance, and would be deleted
   # automatically with it.
   else
      if [ -n "${root_volume}" ]; then
         echo "Waiting for working root volume ${root_volume} to be available"
         aws ec2 wait volume-available \
            --region "${REGION}" \
            --volume-ids "${root_volume}"
         echo "Cleaning up working root volume"
         aws ec2 delete-volume \
            --output text \
            --region "${REGION}" \
            --volume-id "${root_volume}"
         unset root_volume
      fi
      if [ -n "${data_volume}" ]; then
         echo "Waiting for working data volume ${data_volume} to be available"
         aws ec2 wait volume-available \
            --region "${REGION}" \
            --volume-ids "${data_volume}"
         echo "Cleaning up working data volume"
         aws ec2 delete-volume \
            --output text \
            --region "${REGION}" \
            --volume-id "${data_volume}"
         unset data_volume
      fi
   fi

   # Clean up snapshots if we failed to make an AMI from them
   if [ -n "${root_snapshot}" ]; then
      echo "Deleting root snapshot from failed attempt"
      aws ec2 delete-snapshot --snapshot-id "${root_snapshot}"
      unset root_snapshot
   fi
   if [ -n "${data_snapshot}" ]; then
      echo "Deleting data snapshot from failed attempt"
      aws ec2 delete-snapshot --snapshot-id "${data_snapshot}"
      unset data_snapshot
   fi
}

trap 'cleanup' EXIT

block_device_mappings() {
   local root_snapshot="${1:?}"
   local root_volume_size="${2:?}"
   local data_snapshot="${3:?}"
   local data_volume_size="${4:?}"

   cat <<-EOF | jq --compact-output .
	[
	   {
	      "DeviceName": "${ROOT_DEVICE_NAME}",
	      "Ebs": {
	         "SnapshotId": "${root_snapshot}",
	         "VolumeType": "${VOLUME_TYPE}",
	         "VolumeSize": ${root_volume_size},
	         "DeleteOnTermination": true
	      }
	   },
	   {
	      "DeviceName": "${DATA_DEVICE_NAME}",
	      "Ebs": {
	         "SnapshotId": "${data_snapshot}",
	         "VolumeType": "${VOLUME_TYPE}",
	         "VolumeSize": ${data_volume_size},
	         "DeleteOnTermination": true
	      }
	   }
	]
	EOF
}

valid_resource_id() {
   prefix="${1:?}"
   id="${2?}"  # no colon; allow blank so we can use this test before we set a value
   [[ "${id}" =~ ^${prefix}-([a-f0-9]{8}|[a-f0-9]{17})$ ]]
}

# Used to check whether an AMI name is already registered, so we use the
# primary key of owner+name
find_ami() {
   name="${1:?}"
   ami=$(aws ec2 describe-images \
      --output json \
      --region "${REGION}" \
      --owners "self" \
      --filters "Name=name,Values=${name}" \
      | jq --raw-output '.Images[].ImageId')

   if ! valid_resource_id ami "${ami}"; then
      echo "Unable to find AMI ${name}" >&2
      return 1
   fi
   echo "${ami}"
   return 0
}

# Helper to check for errors
check_return() {
   local rc="${1:?}"
   local msg="${2:?}"

   if [ -z "${rc}" ] || [ -z "${msg}" ] || [ -n "${3}" ]; then
      # Developer error, don't continue
      echo '** Usage: check_return RC "message"' >&2
      exit 1
   fi

   if [ "${rc}" -ne 0 ]; then
      echo "*** ${msg}"
      return 1
   fi

   return 0
}

# Helper to conditionally write out attribute if WRITE_OUTPUT_DIR is
# configured.
write_output() {
    local name="$1"
    local value="$2"

    if [[ -z "${WRITE_OUTPUT_DIR}" ]]; then
        return
    fi

    mkdir -p "${WRITE_OUTPUT_DIR}/$(dirname "$name")"
    echo -n "$value" > "${WRITE_OUTPUT_DIR}/${name}"
}

# =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

# Initial setup and checks

parse_args "${@}"

echo "Checking if AMI already exists with name '${NAME}'"
registered_ami="$(find_ami "${NAME}")"
if [ -n "${registered_ami}" ]; then
   echo "Warning! ${registered_ami} ${NAME} already exists in ${REGION}!" >&2
   exit 1
fi

# Determine the size of the images (in G, for EBS)
# 2G      bottlerocket-aws-k8s-1.17-x86_64.img
# 8G      bottlerocket-aws-k8s-1.17-x86_64-data.img
# This is overridden by --root-volume-size and --data-volume-size if you pass those options.
root_image_size=$(du --apparent-size --block-size=G "${ROOT_IMAGE}" | sed -r 's,^([0-9]+)G\t.*,\1,')
if [ ! "${root_image_size}" -gt 0 ]; then
   echo "* Couldn't find the size of the root image!" >&2
   exit 1
fi

ROOT_VOLUME_SIZE="${ROOT_VOLUME_SIZE:-${root_image_size}}"

data_image_size=$(du --apparent-size --block-size=G "${DATA_IMAGE}" | sed -r 's,^([0-9]+)G\t.*,\1,')
if [ ! "${data_image_size}" -gt 0 ]; then
   echo "* Couldn't find the size of the data image!" >&2
   exit 1
fi

DATA_VOLUME_SIZE="${DATA_VOLUME_SIZE:-${DATA_VOLUME_DEFAULT_SIZE}}"
if [ "${data_image_size}" -gt "${DATA_VOLUME_SIZE}" ]; then
   echo "* Found the size of the data image was too large at ${data_image_size}GB, max is ${DATA_VOLUME_SIZE}GB" >&2
   exit 1
fi


# =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

# Start our registration attempts

attempts=0
while true; do
   let attempts+=1
   if [ ${attempts} -gt ${MAX_ATTEMPTS} ]; then
      echo "ERROR! Retry limit (${MAX_ATTEMPTS}) reached!" >&2
      exit 1
   fi

   echo -e "\n* Phase 1: launch a worker instance"

   worker_block_device_mapping=$(cat <<-EOF
	[
	   {
	      "DeviceName": "${ROOT_DEVICE}",
	      "Ebs": {
	         "VolumeSize": ${root_image_size},
	         "DeleteOnTermination": false
	      }
	   },
	   {
	      "DeviceName": "${DATA_DEVICE}",
	      "Ebs": {
	         "VolumeSize": ${data_image_size},
	         "DeleteOnTermination": false
	      }
	   }
	]
	EOF
   )

   echo "Launching worker instance"
   instance=$(aws ec2 run-instances \
      --output json \
      --region "${REGION}" \
      --image-id "${WORKER_AMI}" \
      --tag-specifications 'ResourceType=instance,Tags=[{Key=Name,Value=amiize-worker}]' \
      --instance-type "${INSTANCE_TYPE}" \
      ${SUBNET_ID:+--subnet-id "${SUBNET_ID}"} \
      ${USER_DATA:+--user-data "${USER_DATA}"} \
      ${SECURITY_GROUP_NAME:+--security-groups "${SECURITY_GROUP_NAME}"} \
      ${SECURITY_GROUP_ID:+--security-group-ids "${SECURITY_GROUP_ID}"} \
      --key "${SSH_KEYPAIR}" \
      --block-device-mapping "${worker_block_device_mapping}" \
      | jq --raw-output '.Instances[].InstanceId')

   valid_resource_id i "${instance}"
   check_return ${?} "No instance launched!" || continue
   echo "Launched worker instance ${instance}"

   echo "Waiting for the worker instance to be running"
   tries=0
   status="unknown"
   sleep 20
   while [ "${status}" != "running" ]; do
      echo "Current status: ${status}"
      if [ "${tries}" -ge 10 ]; then
         echo "* Instance didn't start running in allotted time!" >&2
         cleanup
         continue 2
      fi
      let tries+=1

      sleep 6
      status=$(aws ec2 describe-instances \
         --output json \
         --region "${REGION}" \
         --instance-ids "${instance}" \
         | jq --raw-output --exit-status '.Reservations[].Instances[].State.Name')

      check_return ${?} "Couldn't find instance state in describe-instances output!" || continue
   done
   echo "Found status: ${status}"

   # Get the IP to connect to, and the volumes to which we write the images
   echo "Querying host IP and volume"
   json_output=$(aws ec2 describe-instances \
      --output json \
      --region "${REGION}" \
      --instance-ids "${instance}")
   check_return ${?} "Couldn't describe instance!" || { cleanup; continue; }

   jq_host_query=".Reservations[].Instances[].PublicDnsName"
   host=$(echo "${json_output}" | jq --raw-output --exit-status "${jq_host_query}")
   check_return ${?} "Couldn't find hostname in describe-instances output!" || { cleanup; continue; }

   jq_rootvolumeid_query=".Reservations[].Instances[].BlockDeviceMappings[] | select(.DeviceName == \"${ROOT_DEVICE}\") | .Ebs.VolumeId"
   root_volume=$(echo "${json_output}" | jq --raw-output --exit-status "${jq_rootvolumeid_query}")
   check_return ${?} "Couldn't find ebs root-volume-id in describe-instances output!" || { cleanup; continue; }

   jq_datavolumeid_query=".Reservations[].Instances[].BlockDeviceMappings[] | select(.DeviceName == \"${DATA_DEVICE}\") | .Ebs.VolumeId"
   data_volume=$(echo "${json_output}" | jq --raw-output --exit-status "${jq_datavolumeid_query}")
   check_return ${?} "Couldn't find ebs data-volume-id in describe-instances output!" || { cleanup; continue; }

   [ -n "${host}" ] && [ -n "${root_volume}" ] && [ -n "${data_volume}" ]
   check_return ${?} "Couldn't get hostname and volumes from instance description!" || { cleanup; continue; }
   echo "Found hostname '${host}' and root volume '${root_volume}' and data volume '${data_volume}'"

   echo "Waiting for SSH to be accessible"
   tries=0
   sleep 30
   # shellcheck disable=SC2029 disable=SC2086
   while ! ssh ${SSH_OPTS} -o ConnectTimeout=5 "ec2-user@${host}" "test -b ${ROOT_DEVICE} && test -b ${DATA_DEVICE}"; do
      [ "${tries}" -lt 10 ]
      check_return ${?} "* SSH not responding on instance!" || { cleanup; continue 2; }
      sleep 6
      let tries+=1
   done

   # =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

   echo -e "\n* Phase 2: send and write the images"

   echo "Uploading the images to the instance"
   rsync --compress --sparse --rsh="ssh ${SSH_OPTS}" \
      "${ROOT_IMAGE}" "${DATA_IMAGE}" "ec2-user@${host}:${STORAGE}/"
   check_return ${?} "rsync of root and data images to build host failed!" || { cleanup; continue; }
   REMOTE_ROOT_IMAGE="${STORAGE}/$(basename "${ROOT_IMAGE}")"
   REMOTE_DATA_IMAGE="${STORAGE}/$(basename "${DATA_IMAGE}")"

   echo "Writing the images to the volumes"
   # Run the script in a root shell, which requires -tt; -n is a precaution.
   # shellcheck disable=SC2029 disable=SC2086
   ssh ${SSH_OPTS} -tt "ec2-user@${host}" \
      "sudo -n dd conv=sparse conv=fsync bs=256K if=${REMOTE_ROOT_IMAGE} of=${ROOT_DEVICE}"
   check_return ${?} "Writing root image to disk failed!" || { cleanup; continue; }

   # shellcheck disable=SC2029 disable=SC2086
   ssh ${SSH_OPTS} -tt "ec2-user@${host}" \
      "sudo -n dd conv=sparse conv=fsync bs=256K if=${REMOTE_DATA_IMAGE} of=${DATA_DEVICE}"
   check_return ${?} "Writing data image to disk failed!" || { cleanup; continue; }

   # =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

   echo -e "\n* Phase 3: snapshot the volumes"

   echo "Detaching the volumes so we can snapshot them"
   aws ec2 detach-volume \
      --output text \
      --region "${REGION}" \
      --volume-id "${root_volume}"
   check_return ${?} "detach of new root volume failed!" || { cleanup; continue; }

   aws ec2 detach-volume \
      --output text \
      --region "${REGION}" \
      --volume-id "${data_volume}"
   check_return ${?} "detach of new data volume failed!" || { cleanup; continue; }

   echo "Terminating the instance"
   if aws ec2 terminate-instances \
      --output text \
      --region "${REGION}" \
      --instance-ids "${instance}"
   then
      # So the cleanup function doesn't try to stop it
      unset instance
   else
      echo "* Warning: Could not terminate instance!"
      # Don't die though, we got what we want...
   fi

   echo "Waiting for the volumes to be 'available'"
   tries=0
   root_status="unknown"
   data_status="unknown"
   sleep 20
   while [ "${root_status}" != "available" ] || [ "${data_status}" != "available" ]; do
      echo "Current status: root=${root_status}, data=${data_status}"
      [ "${tries}" -lt 20 ]
      check_return ${?} "* Volumes didn't become available in allotted time!" || { cleanup; continue 2; }
      let tries+=1
      sleep 6

      root_status=$(aws ec2 describe-volumes \
         --output json \
         --region "${REGION}" \
         --volume-id "${root_volume}" \
         | jq --raw-output --exit-status '.Volumes[].State')
      check_return ${?} "Couldn't find root volume state in describe-volumes output!" || continue
      data_status=$(aws ec2 describe-volumes \
         --output json \
         --region "${REGION}" \
         --volume-id "${data_volume}" \
         | jq --raw-output --exit-status '.Volumes[].State')
      check_return ${?} "Couldn't find data volume state in describe-volumes output!" || continue
   done
   echo "Found status: root=${root_status}, data=${data_status}"

   # =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

   echo "Snapshotting the volumes so we can create an AMI from them"
   root_snapshot=$(aws ec2 create-snapshot \
      --output json \
      --region "${REGION}" \
      --description "${NAME}" \
      --volume-id "${root_volume}" \
      | jq --raw-output '.SnapshotId')

   valid_resource_id snap "${root_snapshot}"
   check_return ${?} "creating snapshot of new root volume failed!" || { cleanup; continue; }

   data_snapshot=$(aws ec2 create-snapshot \
      --output json \
      --region "${REGION}" \
      --description "${NAME}" \
      --volume-id "${data_volume}" \
      | jq --raw-output '.SnapshotId')

   valid_resource_id snap "${data_snapshot}"
   check_return ${?} "creating snapshot of new data volume failed!" || { cleanup; continue; }

   echo "Waiting for the snapshots to complete"
   tries=0
   root_status="unknown"
   data_status="unknown"
   sleep 20
   while [ "${root_status}" != "completed" ] || [ "${data_status}" != "completed" ]; do
      echo "Current status: root=${root_status}, data=${data_status}"
      [ "${tries}" -lt 75 ]
      check_return ${?} "* Snapshots didn't complete in allotted time!" || { cleanup; continue 2; }
      let tries+=1
      sleep 10

      root_status=$(aws ec2 describe-snapshots \
         --output json \
         --region "${REGION}" \
         --snapshot-ids "${root_snapshot}" \
         | jq --raw-output --exit-status '.Snapshots[].State')
      check_return ${?} "Couldn't find root snapshot state in describe-snapshots output!" || continue
      data_status=$(aws ec2 describe-snapshots \
         --output json \
         --region "${REGION}" \
         --snapshot-ids "${data_snapshot}" \
         | jq --raw-output --exit-status '.Snapshots[].State')
      check_return ${?} "Couldn't find data snapshot state in describe-snapshots output!" || continue
   done
   echo "Found status: root=${root_status}, data=${data_status}"

   echo "Deleting volumes"
   if aws ec2 delete-volume \
      --output text \
      --region "${REGION}" \
      --volume-id "${root_volume}"
   then
      # So the cleanup function doesn't try to delete it
      unset root_volume
   else
      echo "* Warning: Could not delete root volume!"
      # Don't die though, we got what we want...
   fi
   write_output "root_snapshot_id" "$root_snapshot"

   if aws ec2 delete-volume \
      --output text \
      --region "${REGION}" \
      --volume-id "${data_volume}"
   then
      # So the cleanup function doesn't try to delete it
      unset data_volume
   else
      echo "* Warning: Could not delete data volume!"
      # Don't die though, we got what we want...
   fi
   write_output "data_snapshot_id" "$data_snapshot"

   # =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

   echo -e "\n* Phase 4: register the AMI"

   echo "Registering an AMI from the snapshot"
   # shellcheck disable=SC2086
   registered_ami=$(aws --region "${REGION}" ec2 register-image \
      --output text \
      --root-device-name "${ROOT_DEVICE_NAME}" \
      --architecture "${ARCH}" \
      ${SRIOV_FLAG} \
      ${ENA_FLAG} \
      --virtualization-type "${VIRT_TYPE}" \
      --block-device-mappings "$(block_device_mappings \
                                    ${root_snapshot} ${ROOT_VOLUME_SIZE} \
                                    ${data_snapshot} ${DATA_VOLUME_SIZE})" \
      --name "${NAME}" \
      --description "${DESCRIPTION}")
   check_return ${?} "AMI registration failed!" || { cleanup; continue; }

   # So we don't try to delete the snapshots behind our new AMI
   unset root_snapshot data_snapshot

   echo "Registered ${registered_ami}"

   write_output "ami_id" "$registered_ami"

   echo "Waiting for the AMI to appear in a describe query"
   waits=0
   while [ ${waits} -lt 20 ]; do
      if find_ami "${NAME}" >/dev/null; then
         echo "Found AMI ${NAME}: ${registered_ami} in ${REGION}"
         exit 0
      fi
      echo "Waiting a bit more for AMI..."
      sleep 10
      let waits+=1
   done

   echo "Warning: ${registered_ami} doesn't show up in a describe yet; check the EC2 console for further status" >&2
done

echo "No attempts succeeded" >&2
exit 1
