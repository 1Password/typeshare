package proto

import (
    "encoding/json"
)

type AliasTest []SomethingFoo

type Test struct {
	Field1 SomethingFoo `json:"field1"`
	Field2 *SomethingFoo `json:"field2,omitempty"`
}
type Foo string
const (
	FooA Foo = "A"
)
type ParentTypes string
const (
	ParentTypeVariantB ParentTypes = "B"
)
type Parent struct{ 
	Type ParentTypes `json:"type"`
	value interface{}
}

func (p *Parent) UnmarshalJSON(data []byte) error {
	var enum struct {
		Tag    ParentTypes   `json:"type"`
		Content json.RawMessage `json:"value"`
	}
	if err := json.Unmarshal(data, &enum); err != nil {
		return err
	}

	p.Type = enum.Tag
	switch p.Type {
	case ParentTypeVariantB:
		var res SomethingFoo
		p.value = &res

	}
	if err := json.Unmarshal(enum.Content, &p.value); err != nil {
		return err
	}

	return nil
}

func (p Parent) MarshalJSON() ([]byte, error) {
    var enum struct {
		Tag    ParentTypes   `json:"type"`
		Content interface{} `json:"value,omitempty"`
    }
    enum.Tag = p.Type
    enum.Content = p.value
    return json.Marshal(enum)
}

func (p Parent) B() SomethingFoo {
	res, _ := p.value.(*SomethingFoo)
	return *res
}

func NewParentTypeVariantB(content SomethingFoo) Parent {
    return Parent{
        Type: ParentTypeVariantB,
        value: &content,
    }
}

