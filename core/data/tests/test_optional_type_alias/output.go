package proto

import "encoding/json"

type OptionalU32 *uint32

type OptionalU16 *int

type FooBar struct {
	Foo OptionalU32 `json:"foo"`
	Bar OptionalU16 `json:"bar"`
}
