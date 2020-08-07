#!/usr/bin/env bash

# Register partitioned root and data images as an AMI in EC2.
# Only registers with HVM virtualization type and GP2 EBS volume type.

# Image assumptions:
# * Your images are partitioned, and the root image has a bootloader set up as required.
# * Your root image supports SR-IOV (e1000) and ENA networking.

# Environment assumptions:
# * aws-cli is set up (via environment or config) to operate EC2 in the given region.
# * Some required tools are available locally; look just below the constants.
#   * In particular, coldsnap: https://github.com/awslabs/coldsnap

# Example call:
#    bin/amiize.sh \
#       --region us-west-2 \
#       --name bottlerocket-20200727-01 \
#       --arch x86_64 \
#       --root-image build/images/x86_64-aws-k8s-1.17/latest/bottlerocket-aws-k8s-1.17-x86_64.img \
#       --data-image build/images/x86_64-aws-k8s-1.17/latest/bottlerocket-aws-k8s-1.17-x86_64-data.img

# Constants

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

# Check for required tools
for tool in jq aws du coldsnap; do
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
                 --name <DESIRED AMI NAME>
                 --arch <ARCHITECTURE>
                 [ --description "My great AMI" ]
                 [ --root-volume-size 1234 ]
                 [ --data-volume-size 5678 ]
                 [ --write-output-dir output-dir ]

Registers the given images as an AMI in the given EC2 region.

Required:
   --root-image               The image file for the AMI root volume
   --data-image               The image file for the AMI data volume
   --region                   The region to upload to
   --name                     The name under which to register the AMI
   --arch                     The machine architecture of the AMI, e.g. x86_64, arm64

Optional:
   --description              The description attached to the registered AMI (defaults to name)
   --root-volume-size         AMI root volume size in GB (defaults to size of disk image)
   --data-volume-size         AMI data volume size in GB (defaults to size of disk image)
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
         --name ) shift; NAME="${1}" ;;
         --arch ) shift; ARCH="${1}" ;;

         --description ) shift; DESCRIPTION="${1}" ;;
         --root-volume-size ) shift; ROOT_VOLUME_SIZE="${1}" ;;
         --data-volume-size ) shift; DATA_VOLUME_SIZE="${1}" ;;
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

   # Defaults
   if [ -z "${DESCRIPTION}" ] ; then
      DESCRIPTION="${NAME}"
   fi
   # ROOT_VOLUME_SIZE and DATA_VOLUME_SIZE are defaulted below,
   # after we calculate image size
}

cleanup() {
   # Clean up snapshots if we failed to make an AMI from them
   if [ -n "${root_snapshot}" ]; then
      echo "Deleting root snapshot ${root_snapshot} from failed attempt"
      aws ec2 delete-snapshot --snapshot-id "${root_snapshot}"
      unset root_snapshot
   fi
   if [ -n "${data_snapshot}" ]; then
      echo "Deleting data snapshot ${data_snapshot} from failed attempt"
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
      exit 2
   fi

   if [ "${rc}" -ne 0 ]; then
      echo "*** ${msg}"
      exit 1
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

# Main workflow - upload snapshots and register an AMI from them.

root_snapshot="$(coldsnap upload "${ROOT_IMAGE}")"
valid_resource_id snap "${root_snapshot}"
check_return ${?} "creating snapshot of new root volume failed!"

data_snapshot="$(coldsnap upload "${DATA_IMAGE}")"
valid_resource_id snap "${data_snapshot}"
check_return ${?} "creating snapshot of new data volume failed!"

coldsnap wait "${root_snapshot}"
check_return ${?} "failed waiting for root volume availability"
coldsnap wait "${data_snapshot}"
check_return ${?} "failed waiting for data volume availability"

write_output "root_snapshot_id" "$root_snapshot"
write_output "data_snapshot_id" "$data_snapshot"

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
check_return ${?} "AMI registration failed!"

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
