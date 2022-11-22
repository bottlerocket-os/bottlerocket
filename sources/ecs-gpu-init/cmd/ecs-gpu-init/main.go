package main

import (
	"encoding/json"
	"fmt"
	"os"

	"github.com/NVIDIA/go-nvml/pkg/nvml"
)

const (
	// NvidiaGPUInfoFilePath is the file path where GPUs and driver info are saved
	NvidiaGPUInfoFilePath = "/var/lib/ecs/gpu/nvidia-gpu-info.json"
)

func main() {
	if err := generateGPUInfoConfig(); err != nil {
		fmt.Fprintf(os.Stderr, "Got error %v", err)
		os.Exit(1)
	}
}

// GenerateGPUInfoConfig generates the configuration used by the ECS agent
func generateGPUInfoConfig() error {
	if ret := nvml.Init(); ret != nvml.SUCCESS {
		return fmt.Errorf("Failed to initialize NVML, got ret %v", ret)
	}
	defer nvml.Shutdown()
	version, ret := nvml.SystemGetDriverVersion()
	if ret != nvml.SUCCESS {
		return fmt.Errorf("Failed to get version, got ret %v", ret)
	}
	gpuIDs, err := getGPUDeviceIDs()
	if err != nil {
		return err
	}

	return writeGPUInfo(version, gpuIDs)
}

// getGPUDeviceIDs returns the UUIDs of all the NVIDIA GPUs
func getGPUDeviceIDs() ([]string, error) {
	count, ret := nvml.DeviceGetCount()
	if ret != nvml.SUCCESS {
		return nil, fmt.Errorf("Failed to get device count, got ret %v", ret)
	}
	var (
		gpuIDs []string
		errors []error
	)
	for i := 0; i < count; i++ {
		uuid, err := getDeviceUUID(i)
		if err != nil {
			errors = append(errors, err)
			continue
		}
		gpuIDs = append(gpuIDs, uuid)
	}
	if len(errors) > 0 {
		return nil, fmt.Errorf("Found errors while initializing devices: %s", errors)
	}
	return gpuIDs, nil
}

// NvidiaInfo represents the configuration required by the ECS agent to assign GPUs to tasks
// For reference:
//
//	https://github.com/aws/amazon-ecs-init/blob/master/ecs-init/gpu/nvidia_gpu_manager.go#L42
type NvidiaInfo struct {
	DriverVersion string
	GPUIDs        []string
}

// writeGPUInfo writes the GPU info config file using the passed `version` and `gpuIDs`
func writeGPUInfo(version string, gpuIDs []string) error {
	nvidiaInfo := NvidiaInfo{DriverVersion: version, GPUIDs: gpuIDs}
	nvidiaInfoJSON, err := json.Marshal(nvidiaInfo)
	if err != nil {
		return err
	}

	return os.WriteFile(NvidiaGPUInfoFilePath, nvidiaInfoJSON, 0700)
}

// getDeviceUUID returns the UUID for the GPU at index `id`
func getDeviceUUID(idx int) (string, error) {
	d, ret := nvml.DeviceGetHandleByIndex(idx)
	if ret != nvml.SUCCESS {
		return "", fmt.Errorf("Failed to get device at index %d, got ret %v", idx, ret)
	}
	uuid, ret := d.GetUUID()
	if ret != nvml.SUCCESS {
		return "", fmt.Errorf("Failed to get UUID for device at index %d, got ret %v", idx, ret)
	}

	return uuid, nil
}
