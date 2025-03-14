package proto

import "encoding/json"

// Generated type representing the anonymous struct variant `Us` of the `AutofilledBy` Rust enum
type AutofilledByUsInner struct {
	// The UUID for the fill
	Uuid string `json:"uuid"`
}
// Generated type representing the anonymous struct variant `SomethingElse` of the `AutofilledBy` Rust enum
type AutofilledBySomethingElseInner struct {
	// The UUID for the fill
	Uuid string `json:"uuid"`
}
// Enum keeping track of who autofilled a field
type AutofilledByTypes string
const (
	// This field was autofilled by us
	AutofilledByTypeVariantUs AutofilledByTypes = "Us"
	// Something else autofilled this field
	AutofilledByTypeVariantSomethingElse AutofilledByTypes = "SomethingElse"
)
type AutofilledBy struct{ 
	Type AutofilledByTypes `json:"type"`
	content interface{}
}

func (a *AutofilledBy) UnmarshalJSON(data []byte) error {
	var enum struct {
		Tag    AutofilledByTypes   `json:"type"`
		Content json.RawMessage `json:"content"`
	}
	if err := json.Unmarshal(data, &enum); err != nil {
		return err
	}

	a.Type = enum.Tag
	switch a.Type {
	case AutofilledByTypeVariantUs:
		var res AutofilledByUsInner
		a.content = &res
	case AutofilledByTypeVariantSomethingElse:
		var res AutofilledBySomethingElseInner
		a.content = &res

	}
	if err := json.Unmarshal(enum.Content, &a.content); err != nil {
		return err
	}

	return nil
}

func (a AutofilledBy) MarshalJSON() ([]byte, error) {
    var enum struct {
		Tag    AutofilledByTypes   `json:"type"`
		Content interface{} `json:"content,omitempty"`
    }
    enum.Tag = a.Type
    enum.Content = a.content
    return json.Marshal(enum)
}

func (a AutofilledBy) Us() *AutofilledByUsInner {
	res, _ := a.content.(*AutofilledByUsInner)
	return res
}
func (a AutofilledBy) SomethingElse() *AutofilledBySomethingElseInner {
	res, _ := a.content.(*AutofilledBySomethingElseInner)
	return res
}

func NewAutofilledByTypeVariantUs(content *AutofilledByUsInner) AutofilledBy {
    return AutofilledBy{
        Type: AutofilledByTypeVariantUs,
        content: content,
    }
}
func NewAutofilledByTypeVariantSomethingElse(content *AutofilledBySomethingElseInner) AutofilledBy {
    return AutofilledBy{
        Type: AutofilledByTypeVariantSomethingElse,
        content: content,
    }
}

