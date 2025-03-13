package proto

import "encoding/json"

// A struct with no target_os. Should be generated when
// we use --target-os.
type AlwaysAccept struct {
}
type DefinedTwice struct {
	Field1 string `json:"field1"`
}
type Excluded struct {
}
type ManyStruct struct {
}
type MultipleTargets struct {
}
type NestedNotTarget1 struct {
}
type OtherExcluded struct {
}
type AlwaysAcceptEnum string
const (
	AlwaysAcceptEnumVariant1 AlwaysAcceptEnum = "Variant1"
	AlwaysAcceptEnumVariant2 AlwaysAcceptEnum = "Variant2"
)
type SomeEnum string
const (
)
// Generated type representing the anonymous struct variant `Variant7` of the `TestEnum` Rust enum
type TestEnumVariant7Inner struct {
	Field1 string `json:"field1"`
}
// Generated type representing the anonymous struct variant `Variant9` of the `TestEnum` Rust enum
type TestEnumVariant9Inner struct {
	Field2 string `json:"field2"`
}
type TestEnumTypes string
const (
	TestEnumTypeVariantVariant1 TestEnumTypes = "Variant1"
	TestEnumTypeVariantVariant5 TestEnumTypes = "Variant5"
	TestEnumTypeVariantVariant7 TestEnumTypes = "Variant7"
	TestEnumTypeVariantVariant8 TestEnumTypes = "Variant8"
	TestEnumTypeVariantVariant9 TestEnumTypes = "Variant9"
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
	case TestEnumTypeVariantVariant1:
		return nil
	case TestEnumTypeVariantVariant5:
		return nil
	case TestEnumTypeVariantVariant7:
		var res TestEnumVariant7Inner
		t.content = &res
	case TestEnumTypeVariantVariant8:
		return nil
	case TestEnumTypeVariantVariant9:
		var res TestEnumVariant9Inner
		t.content = &res

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
func (t TestEnum) Variant9() *TestEnumVariant9Inner {
	res, _ := t.content.(*TestEnumVariant9Inner)
	return res
}

func NewTestEnumTypeVariantVariant1() TestEnum {
    return TestEnum{
        Type: TestEnumTypeVariantVariant1,
    }
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
func NewTestEnumTypeVariantVariant9(content *TestEnumVariant9Inner) TestEnum {
    return TestEnum{
        Type: TestEnumTypeVariantVariant9,
        content: content,
    }
}

