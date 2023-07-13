package proto

import "encoding/json"

// This is a comment.
type ColorsTypes string
const (
	ColorsTypeVariantRed ColorsTypes = "Red"
	ColorsTypeVariantBlue ColorsTypes = "Blue"
	ColorsTypeVariantGreen ColorsTypes = "Green"
)
type Colors struct{ 
	Type ColorsTypes `json:"type"`
	content interface{}
}

func (c *Colors) UnmarshalJSON(data []byte) error {
	var enum struct {
		Tag    ColorsTypes   `json:"type"`
		Content json.RawMessage `json:"content"`
	}
	if err := json.Unmarshal(data, &enum); err != nil {
		return err
	}

	c.Type = enum.Tag
	switch c.Type {
	case ColorsTypeVariantRed:
		return nil
	case ColorsTypeVariantBlue:
		return nil
	case ColorsTypeVariantGreen:
		var res string
		c.content = &res

	}
	if err := json.Unmarshal(enum.Content, &c.content); err != nil {
		return err
	}

	return nil
}

func (c Colors) MarshalJSON() ([]byte, error) {
    var enum struct {
		Tag    ColorsTypes   `json:"type"`
		Content interface{} `json:"content,omitempty"`
    }
    enum.Tag = c.Type
    enum.Content = c.content
    return json.Marshal(enum)
}

func (c Colors) Green() string {
	res, _ := c.content.(*string)
	return *res
}

func NewColorsTypeVariantRed() Colors {
    return Colors{
        Type: ColorsTypeVariantRed,
    }
}
func NewColorsTypeVariantBlue() Colors {
    return Colors{
        Type: ColorsTypeVariantBlue,
    }
}
func NewColorsTypeVariantGreen(content string) Colors {
    return Colors{
        Type: ColorsTypeVariantGreen,
        content: &content,
    }
}

