package proto

import "encoding/json"

type OverrideStruct struct {
	FieldToOverride uint `json:"fieldToOverride"`
}
// Generated type representing the anonymous struct variant `AnonymousStructVariant` of the `OverrideEnum` Rust enum
type OverrideEnumAnonymousStructVariantInner struct {
	FieldToOverride uint `json:"fieldToOverride"`
}
type OverrideEnumTypes string
const (
	OverrideEnumTypeVariantUnitVariant OverrideEnumTypes = "UnitVariant"
	OverrideEnumTypeVariantTupleVariant OverrideEnumTypes = "TupleVariant"
	OverrideEnumTypeVariantAnonymousStructVariant OverrideEnumTypes = "AnonymousStructVariant"
)
type OverrideEnum struct{ 
	Type OverrideEnumTypes `json:"type"`
	content interface{}
}

func (o *OverrideEnum) UnmarshalJSON(data []byte) error {
	var enum struct {
		Tag    OverrideEnumTypes   `json:"type"`
		Content json.RawMessage `json:"content"`
	}
	if err := json.Unmarshal(data, &enum); err != nil {
		return err
	}

	o.Type = enum.Tag
	switch o.Type {
	case OverrideEnumTypeVariantUnitVariant:
		return nil
	case OverrideEnumTypeVariantTupleVariant:
		var res string
		o.content = &res
	case OverrideEnumTypeVariantAnonymousStructVariant:
		var res OverrideEnumAnonymousStructVariantInner
		o.content = &res

	}
	if err := json.Unmarshal(enum.Content, &o.content); err != nil {
		return err
	}

	return nil
}

func (o OverrideEnum) MarshalJSON() ([]byte, error) {
    var enum struct {
		Tag    OverrideEnumTypes   `json:"type"`
		Content interface{} `json:"content,omitempty"`
    }
    enum.Tag = o.Type
    enum.Content = o.content
    return json.Marshal(enum)
}

func (o OverrideEnum) TupleVariant() string {
	res, _ := o.content.(*string)
	return *res
}
func (o OverrideEnum) AnonymousStructVariant() *OverrideEnumAnonymousStructVariantInner {
	res, _ := o.content.(*OverrideEnumAnonymousStructVariantInner)
	return res
}

func NewOverrideEnumTypeVariantUnitVariant() OverrideEnum {
    return OverrideEnum{
        Type: OverrideEnumTypeVariantUnitVariant,
    }
}
func NewOverrideEnumTypeVariantTupleVariant(content string) OverrideEnum {
    return OverrideEnum{
        Type: OverrideEnumTypeVariantTupleVariant,
        content: &content,
    }
}
func NewOverrideEnumTypeVariantAnonymousStructVariant(content *OverrideEnumAnonymousStructVariantInner) OverrideEnum {
    return OverrideEnum{
        Type: OverrideEnumTypeVariantAnonymousStructVariant,
        content: content,
    }
}

