package proto

import "encoding/json"

type Video struct {
	Tags []Tag `json:"tags"`
}
