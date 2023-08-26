package proto

import "encoding/json"

type DisallowedType struct {
	DisallowedType uint64 `json:"disallowed_type"`
	AnotherDisallowedType int64 `json:"another_disallowed_type"`
	DisallowedTypeSerdeWith uint64 `json:"disallowed_type_serde_with"`
}
