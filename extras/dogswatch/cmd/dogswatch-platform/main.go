package main

import (
	"fmt"

	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/marker"
)

func main() {
	fmt.Println(marker.PlatformVersionBuild)
}
