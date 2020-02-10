package cache

import (
	"time"

	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/intent"

	"github.com/karlseguin/ccache"
)

const (
	cacheTimeout = time.Second * 15
)

// LastCache provides access to the last cached Intent that came from the same
// source of the provided Intent.
type LastCache interface {
	Last(*intent.Intent) *intent.Intent
	Record(*intent.Intent)
}

type lastCache struct {
	cache *ccache.Cache
}

// NewLastCache creates a general cache suitable for storing and retrieving the
// last observed Intent given its source.
func NewLastCache() LastCache {
	return &lastCache{
		cache: ccache.New(ccache.Configure().MaxSize(1000).ItemsToPrune(100)),
	}
}

// Last returns the last intent to be sent through.
func (i *lastCache) Last(in *intent.Intent) *intent.Intent {
	if in == nil {
		return nil
	}
	val := i.cache.Get(in.GetName())
	if val == nil {
		return nil
	}
	if val.Expired() {
		return nil
	}
	lastCachedIntent, ok := val.Value().(*intent.Intent)
	if !ok {
		return nil
	}

	// TODO: possibly extend a cached item
	// val.Extend(cacheExtension)

	// Copy to protect against misuse of cached in-memory Intent.
	return lastCachedIntent.Clone()
}

// Record caches the provided Intent as the most recent Intent handled for a
// given intent.
func (i *lastCache) Record(in *intent.Intent) {
	if in == nil {
		return
	}
	i.cache.Set(in.GetName(), in.Clone(), cacheTimeout)
}
