package proto

import "encoding/json"

type A struct {
	Field uint32 `json:"field"`
}
type AB struct {
	Field uint32 `json:"field"`
}
type ABC struct {
	Field uint32 `json:"field"`
}
type OutsideOfModules struct {
	Field uint32 `json:"field"`
}
