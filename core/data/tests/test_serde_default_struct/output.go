package proto

import "encoding/json"

type Foo struct {
	Bar *bool `json:"bar,omitempty"`
}
