package proto

import "encoding/json"

type MultipleTargets struct {
}
type SomeEnum string
const (
)
// Generated type representing the anonymous struct variant `Variant7` of the `TestEnum` Rust enum
type TestEnumVariant7Inner struct {
}
type TestEnumTypes string
const (
	TestEnumTypeVariantVariant5 TestEnumTypes = "Variant5"
	TestEnumTypeVariantVariant7 TestEnumTypes = "Variant7"
	TestEnumTypeVariantVariant8 TestEnumTypes = "Variant8"
)
type TestEnum struct{ 
	Type TestEnumTypes `json:"type"`
	content interface{}
}

func (t *TestEnum) UnmarshalJSON(data []byte) error {
	var enum struct {
		Tag    TestEnumTypes   `json:"type"`
		Content json.RawMessage `json:"content"`
	}
	if err := json.Unmarshal(data, &enum); err != nil {
		return err
	}

	t.Type = enum.Tag
	switch t.Type {
	case TestEnumTypeVariantVariant5:
		return nil
	case TestEnumTypeVariantVariant7:
		var res TestEnumVariant7Inner
		t.content = &res
	case TestEnumTypeVariantVariant8:
		return nil

	}
	if err := json.Unmarshal(enum.Content, &t.content); err != nil {
		return err
	}

	return nil
}

func (t TestEnum) MarshalJSON() ([]byte, error) {
    var enum struct {
		Tag    TestEnumTypes   `json:"type"`
		Content interface{} `json:"content,omitempty"`
    }
    enum.Tag = t.Type
    enum.Content = t.content
    return json.Marshal(enum)
}

func (t TestEnum) Variant7() *TestEnumVariant7Inner {
	res, _ := t.content.(*TestEnumVariant7Inner)
	return res
}

func NewTestEnumTypeVariantVariant5() TestEnum {
    return TestEnum{
        Type: TestEnumTypeVariantVariant5,
    }
}
func NewTestEnumTypeVariantVariant7(content *TestEnumVariant7Inner) TestEnum {
    return TestEnum{
        Type: TestEnumTypeVariantVariant7,
        content: content,
    }
}
func NewTestEnumTypeVariantVariant8() TestEnum {
    return TestEnum{
        Type: TestEnumTypeVariantVariant8,
    }
}

