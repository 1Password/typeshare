package proto

import "encoding/json"

type Bar string

type Foo struct {
	Bar Bar `json:"bar"`
}
