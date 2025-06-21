package proto

import "encoding/json"

type PointerSizedType struct {
	Unsigned uint64 `json:"unsigned"`
	Signed int64 `json:"signed"`
}
