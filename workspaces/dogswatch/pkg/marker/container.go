package marker

// Container is a collection of generic markers that represent Annotations and
// Labels.
type Container interface {
	// GetAnnotations retrieves the markers that represent a set of annotations.
	GetAnnotations() map[string]string
	// GetLabels retrieves the markers that represent a set of labels.
	GetLabels() map[string]string
}

// WriteContainer is a container that may be written into in addition to being a
// Container.
type WriteContainer interface {
	Container
	SetAnnotations(map[string]string)
	SetLabels(map[string]string)
}

// Overwrite into a container from another in place.
func OverwriteFrom(from Container, into WriteContainer) {
	fromA := from.GetAnnotations()
	intoA := into.GetAnnotations()
	for k := range fromA {
		intoA[k] = fromA[k]
	}
	fromL := from.GetLabels()
	intoL := into.GetLabels()
	for k := range fromA {
		intoL[k] = fromL[k]
	}

	into.SetAnnotations(intoA)
	into.SetLabels(intoL)
}
