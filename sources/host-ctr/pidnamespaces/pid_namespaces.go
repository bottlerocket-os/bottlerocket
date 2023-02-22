package pidnamespaces

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/gofrs/flock"
	"github.com/pkg/errors"
	"golang.org/x/sys/unix"
)

const (
	lockfile     string = "/run/lock/host-ctr-pid-ns.lock"
	manifestPath string = "/etc/host-ctr-namespaces/pid-namespaces.json"
)

// HostCtrPidNs represents the list of PID namespaces
type HostCtrPidNs struct {
	PidNamespaces []int64 `json:"pid-namespaces,omitempty"`
}

// GetTaskPidNs returns the PID NS associated with the container task PID or error on failure
func GetTaskPidNs(pid uint32) (int64, error) {
	pidNsFile, err := os.Open(fmt.Sprintf("/proc/%d/ns/pid", pid))
	if err != nil {
		return -1, fmt.Errorf("failed to open %d PID namespace file", pid)
	}
	pidNsFd := int(pidNsFile.Fd())

	// Get file descriptor that refers to a parent namespace, ioctl() request '_IO(NSIO, 0x2)'
	parentPidNsFd, err := unix.IoctlRetInt(pidNsFd, unix.NS_GET_PARENT)
	if parentPidNsFd == -1 && err == unix.EPERM {
		// If we can't access the parent PID namespace, this means the current container task is sharing the root PID
		// namespace. We don't want to track the root PID namespace in the manifest, so we return -1.
		return -1, err
	} else if err != nil {
		return -1, errors.Wrap(err, fmt.Sprintf("ioctl failed to 'NS_GET_PARENTNS' for '%s'", pidNsFile.Name()))
	}

	// The PID namespace is represented by the inode number of the task's PID namespace file
	var sb unix.Stat_t
	if err := unix.Fstat(pidNsFd, &sb); err != nil {
		return -1, errors.Wrap(err, "failed to stat task PID namespace fd")
	}

	return int64(sb.Ino), nil
}

// AddPidNsToManifest takes a given PID's namespace and adds it to the internal manifest
func AddPidNsToManifest(pidNs int64) (err error) {
	fileLock := flock.New(lockfile)
	lockCtx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	ok, err := fileLock.TryLockContext(lockCtx, 500*time.Millisecond)
	if err != nil && !ok {
		return errors.Wrap(err, "acquiring lock for writing to PID NS manifest failed with error")
	}
	defer func() {
		if err = fileLock.Unlock(); err != nil {
			err = errors.Wrap(err, "failed to release lock for writing to PID NS manifest")
		}
	}()

	// Add task PID NS to manifest
	hostCtrPidNs, err := readManifest(manifestPath)
	if err != nil {
		return err
	}
	for _, ele := range hostCtrPidNs.PidNamespaces {
		// If it already exists in the manifest list for whatever reason, we can just return without doing anything.
		if ele == pidNs {
			return nil
		}
	}
	hostCtrPidNs.PidNamespaces = append(hostCtrPidNs.PidNamespaces, pidNs)
	if err = writeManifest(&hostCtrPidNs, manifestPath); err != nil {
		return err
	}

	return nil
}

// RemovePidNsFromManifest takes a PID's namespace and removes it from the internal manifest
func RemovePidNsFromManifest(pidNs int64) (err error) {
	fileLock := flock.New(lockfile)
	lockCtx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	ok, err := fileLock.TryLockContext(lockCtx, 500*time.Millisecond)
	if err != nil && !ok {
		return errors.Wrap(err, "acquiring lock for writing to PID NS manifest failed with error")
	}
	defer func() {
		if err = fileLock.Unlock(); err != nil {
			err = errors.Wrap(err, "failed to release lock for writing to PID NS manifest")
		}
	}()

	// Remove task PID NS from manifest
	hostCtrPidNs, err := readManifest(manifestPath)
	if err != nil {
		return err
	}

	for i, ele := range hostCtrPidNs.PidNamespaces {
		// Remove PID NS from list
		if ele == pidNs {
			hostCtrPidNs.PidNamespaces = append(hostCtrPidNs.PidNamespaces[:i], hostCtrPidNs.PidNamespaces[i+1:]...)
		}
	}
	if err = writeManifest(&hostCtrPidNs, manifestPath); err != nil {
		return err
	}
	return nil
}

func readManifest(manifestPath string) (HostCtrPidNs, error) {
	manifest, err := os.OpenFile(manifestPath, os.O_RDWR|os.O_CREATE, 0644)
	if err != nil {
		return HostCtrPidNs{}, errors.Wrap(err, "failed to open PID NS manifest file")
	}
	defer manifest.Close()

	manifestStat, err := manifest.Stat()
	if err != nil {
		return HostCtrPidNs{}, errors.Wrap(err, "failed to stat PID NS manifest file")
	}
	raw := make([]byte, manifestStat.Size())
	if _, err = manifest.Read(raw); err != nil {
		return HostCtrPidNs{}, errors.Wrap(err, "failed to read PID NS manifest file")
	}
	hostCtrPidNsList := HostCtrPidNs{}
	if len(raw) != 0 {
		err = json.Unmarshal(raw, &hostCtrPidNsList)
		if err != nil {
			return HostCtrPidNs{}, errors.Wrap(err, "failed to unmarshal PID NS manifest")
		}
	}
	return hostCtrPidNsList, nil
}

// writeManifest writes the list of host-container PID NS to the open manifest file
func writeManifest(hostCtrPidNsList *HostCtrPidNs, manifestPath string) error {
	newData, err := json.Marshal(*hostCtrPidNsList)
	if err != nil {
		return errors.Wrap(err, "failed to marshal PID NS list to JSON")
	}
	tempFile, err := os.CreateTemp(filepath.Dir(manifestPath), "temp-pidns-*.json")
	if err != nil {
		return errors.Wrap(err, "failed to create temporary PID NS file")
	}
	defer tempFile.Close()

	if _, err = tempFile.Write(newData); err != nil {
		return errors.Wrap(err, "failed to write to temporary PID NS file")
	}

	// Persist the temporary PID NS file to the PID NS manifest path to ensure atomicity
	if err = os.Rename(tempFile.Name(), manifestPath); err != nil {
		return errors.Wrap(err, "failed to persist temporary PID NS file to manifest path")
	}
	return nil
}
