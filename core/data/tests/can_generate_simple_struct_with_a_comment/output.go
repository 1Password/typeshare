package proto

import "encoding/json"

type Location struct {
}
// This is a comment.
type Person struct {
	// This is another comment
	Name string `json:"name"`
	Age uint8 `json:"age"`
	Info *string `json:"info,omitempty"`
	Emails []string `json:"emails"`
	Location Location `json:"location"`
}
