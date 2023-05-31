package proto

import (
    "encoding/json"
    "time"
)

type Foo struct {
	Bar *bool `json:"bar,omitempty"`
}
