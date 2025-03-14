package proto

import "encoding/json"

// This is a comment.
type ArcyColors struct {
	Red int `json:"red"`
	Blue string `json:"blue"`
	Green []string `json:"green"`
}
// This is a comment.
type CellyColors struct {
	Red string `json:"red"`
	Blue []string `json:"blue"`
}
// This is a comment.
type CowyColors struct {
	Lifetime string `json:"lifetime"`
}
// This is a comment.
type LockyColors struct {
	Red string `json:"red"`
}
// This is a comment.
type MutexyColors struct {
	Blue []string `json:"blue"`
	Green string `json:"green"`
}
// This is a comment.
type RcyColors struct {
	Red string `json:"red"`
	Blue []string `json:"blue"`
	Green string `json:"green"`
}
// This is a comment.
type BoxyColorsTypes string
const (
	BoxyColorsTypeVariantRed BoxyColorsTypes = "Red"
	BoxyColorsTypeVariantBlue BoxyColorsTypes = "Blue"
	BoxyColorsTypeVariantGreen BoxyColorsTypes = "Green"
)
type BoxyColors struct{ 
	Type BoxyColorsTypes `json:"type"`
	content interface{}
}

func (b *BoxyColors) UnmarshalJSON(data []byte) error {
	var enum struct {
		Tag    BoxyColorsTypes   `json:"type"`
		Content json.RawMessage `json:"content"`
	}
	if err := json.Unmarshal(data, &enum); err != nil {
		return err
	}

	b.Type = enum.Tag
	switch b.Type {
	case BoxyColorsTypeVariantRed:
		return nil
	case BoxyColorsTypeVariantBlue:
		return nil
	case BoxyColorsTypeVariantGreen:
		var res string
		b.content = &res

	}
	if err := json.Unmarshal(enum.Content, &b.content); err != nil {
		return err
	}

	return nil
}

func (b BoxyColors) MarshalJSON() ([]byte, error) {
    var enum struct {
		Tag    BoxyColorsTypes   `json:"type"`
		Content interface{} `json:"content,omitempty"`
    }
    enum.Tag = b.Type
    enum.Content = b.content
    return json.Marshal(enum)
}

func (b BoxyColors) Green() string {
	res, _ := b.content.(*string)
	return *res
}

func NewBoxyColorsTypeVariantRed() BoxyColors {
    return BoxyColors{
        Type: BoxyColorsTypeVariantRed,
    }
}
func NewBoxyColorsTypeVariantBlue() BoxyColors {
    return BoxyColors{
        Type: BoxyColorsTypeVariantBlue,
    }
}
func NewBoxyColorsTypeVariantGreen(content string) BoxyColors {
    return BoxyColors{
        Type: BoxyColorsTypeVariantGreen,
        content: &content,
    }
}

