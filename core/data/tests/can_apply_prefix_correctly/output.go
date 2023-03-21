package proto

import "encoding/json"

type ItemDetailsFieldValue struct {
	Hello string `json:"hello"`
}
type AdvancedColorsTs string
const (
	AdvancedColorsTVariantString AdvancedColorsTs = "String"
	AdvancedColorsTVariantNumber AdvancedColorsTs = "Number"
	AdvancedColorsTVariantNumberArray AdvancedColorsTs = "NumberArray"
	AdvancedColorsTVariantReallyCoolType AdvancedColorsTs = "ReallyCoolType"
	AdvancedColorsTVariantArrayReallyCoolType AdvancedColorsTs = "ArrayReallyCoolType"
	AdvancedColorsTVariantDictionaryReallyCoolType AdvancedColorsTs = "DictionaryReallyCoolType"
)
type AdvancedColors struct{ 
	T AdvancedColorsTs `json:"t"`
	c interface{}
}

func (a *AdvancedColors) UnmarshalJSON(data []byte) error {
	var enum struct {
		Tag    AdvancedColorsTs   `json:"t"`
		Content json.RawMessage `json:"c"`
	}
	if err := json.Unmarshal(data, &enum); err != nil {
		return err
	}

	a.T = enum.Tag
	switch a.T {
	case AdvancedColorsTVariantString:
		var res string
		a.c = &res
	case AdvancedColorsTVariantNumber:
		var res int
		a.c = &res
	case AdvancedColorsTVariantNumberArray:
		var res []int
		a.c = &res
	case AdvancedColorsTVariantReallyCoolType:
		var res ItemDetailsFieldValue
		a.c = &res
	case AdvancedColorsTVariantArrayReallyCoolType:
		var res []ItemDetailsFieldValue
		a.c = &res
	case AdvancedColorsTVariantDictionaryReallyCoolType:
		var res map[string]ItemDetailsFieldValue
		a.c = &res

	}
	if err := json.Unmarshal(enum.Content, &a.c); err != nil {
		return err
	}

	return nil
}

func (a AdvancedColors) MarshalJSON() ([]byte, error) {
    var enum struct {
		Tag    AdvancedColorsTs   `json:"t"`
		Content interface{} `json:"c,omitempty"`
    }
    enum.Tag = a.T
    enum.Content = a.c
    return json.Marshal(enum)
}

func (a AdvancedColors) String() string {
	res, _ := a.c.(*string)
	return *res
}
func (a AdvancedColors) Number() int {
	res, _ := a.c.(*int)
	return *res
}
func (a AdvancedColors) NumberArray() []int {
	res, _ := a.c.(*[]int)
	return *res
}
func (a AdvancedColors) ReallyCoolType() *ItemDetailsFieldValue {
	res, _ := a.c.(*ItemDetailsFieldValue)
	return res
}
func (a AdvancedColors) ArrayReallyCoolType() []ItemDetailsFieldValue {
	res, _ := a.c.(*[]ItemDetailsFieldValue)
	return *res
}
func (a AdvancedColors) DictionaryReallyCoolType() map[string]ItemDetailsFieldValue {
	res, _ := a.c.(*map[string]ItemDetailsFieldValue)
	return *res
}

func NewAdvancedColorsTVariantString(content string) AdvancedColors {
    return AdvancedColors{
        T: AdvancedColorsTVariantString,
        c: &content,
    }
}
func NewAdvancedColorsTVariantNumber(content int) AdvancedColors {
    return AdvancedColors{
        T: AdvancedColorsTVariantNumber,
        c: &content,
    }
}
func NewAdvancedColorsTVariantNumberArray(content []int) AdvancedColors {
    return AdvancedColors{
        T: AdvancedColorsTVariantNumberArray,
        c: &content,
    }
}
func NewAdvancedColorsTVariantReallyCoolType(content *ItemDetailsFieldValue) AdvancedColors {
    return AdvancedColors{
        T: AdvancedColorsTVariantReallyCoolType,
        c: content,
    }
}
func NewAdvancedColorsTVariantArrayReallyCoolType(content []ItemDetailsFieldValue) AdvancedColors {
    return AdvancedColors{
        T: AdvancedColorsTVariantArrayReallyCoolType,
        c: &content,
    }
}
func NewAdvancedColorsTVariantDictionaryReallyCoolType(content map[string]ItemDetailsFieldValue) AdvancedColors {
    return AdvancedColors{
        T: AdvancedColorsTVariantDictionaryReallyCoolType,
        c: &content,
    }
}

