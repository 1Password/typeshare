package proto

import (
    "encoding/json"
	"time"
)

type Foo struct {
	Time time.Time `json:"time"`
	Time2 time.Time `json:"time2"`
	Time3 time.Time `json:"time3"`
	Bytes []byte `json:"bytes"`
	Bytes2 []byte `json:"bytes2"`
}
