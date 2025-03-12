package proto

import (
    "encoding/json"
    "time"
)

type Video struct {
	Tags []Tag `json:"tags"`
}
