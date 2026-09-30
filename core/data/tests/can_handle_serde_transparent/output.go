package proto

import "encoding/json"

type Card struct {
	Name string `json:"name"`
}
type Cards []Card

type Player struct {
	Name string `json:"name"`
}
type Players []Player

type Pool Cards

