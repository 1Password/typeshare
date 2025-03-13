package proto

import "encoding/json"

// This is a comment.
type Things struct {
	Bla string `json:"bla"`
	SomeLabel *string `json:"label,omitempty"`
	LabelLeft *string `json:"label-left,omitempty"`
}
