package proto

import "encoding/json"

type EditItemViewModelSaveRequest struct {
	Context string `json:"context"`
	Values []EditItemSaveValue `json:"values"`
	FillAction *AutoFillItemActionRequest `json:"fill_action,omitempty"`
}
