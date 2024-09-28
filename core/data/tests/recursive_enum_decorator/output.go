package proto

import "encoding/json"

// Generated type representing the anonymous struct variant `Exactly` of the `MoreOptions` Rust enum
type MoreOptionsExactlyInner struct {
	Config string `json:"config"`
}
// Generated type representing the anonymous struct variant `Built` of the `MoreOptions` Rust enum
type MoreOptionsBuiltInner struct {
	Top MoreOptions `json:"top"`
}
type MoreOptionsTypes string
const (
	MoreOptionsTypeVariantNews MoreOptionsTypes = "News"
	MoreOptionsTypeVariantExactly MoreOptionsTypes = "Exactly"
	MoreOptionsTypeVariantBuilt MoreOptionsTypes = "Built"
)
type MoreOptions struct{ 
	Type MoreOptionsTypes `json:"type"`
	content interface{}
}

func (m *MoreOptions) UnmarshalJSON(data []byte) error {
	var enum struct {
		Tag    MoreOptionsTypes   `json:"type"`
		Content json.RawMessage `json:"content"`
	}
	if err := json.Unmarshal(data, &enum); err != nil {
		return err
	}

	m.Type = enum.Tag
	switch m.Type {
	case MoreOptionsTypeVariantNews:
		var res bool
		m.content = &res
	case MoreOptionsTypeVariantExactly:
		var res MoreOptionsExactlyInner
		m.content = &res
	case MoreOptionsTypeVariantBuilt:
		var res MoreOptionsBuiltInner
		m.content = &res

	}
	if err := json.Unmarshal(enum.Content, &m.content); err != nil {
		return err
	}

	return nil
}

func (m MoreOptions) MarshalJSON() ([]byte, error) {
    var enum struct {
		Tag    MoreOptionsTypes   `json:"type"`
		Content interface{} `json:"content,omitempty"`
    }
    enum.Tag = m.Type
    enum.Content = m.content
    return json.Marshal(enum)
}

func (m MoreOptions) News() bool {
	res, _ := m.content.(*bool)
	return *res
}
func (m MoreOptions) Exactly() *MoreOptionsExactlyInner {
	res, _ := m.content.(*MoreOptionsExactlyInner)
	return res
}
func (m MoreOptions) Built() *MoreOptionsBuiltInner {
	res, _ := m.content.(*MoreOptionsBuiltInner)
	return res
}

func NewMoreOptionsTypeVariantNews(content bool) MoreOptions {
    return MoreOptions{
        Type: MoreOptionsTypeVariantNews,
        content: &content,
    }
}
func NewMoreOptionsTypeVariantExactly(content *MoreOptionsExactlyInner) MoreOptions {
    return MoreOptions{
        Type: MoreOptionsTypeVariantExactly,
        content: content,
    }
}
func NewMoreOptionsTypeVariantBuilt(content *MoreOptionsBuiltInner) MoreOptions {
    return MoreOptions{
        Type: MoreOptionsTypeVariantBuilt,
        content: content,
    }
}

type OptionsTypes string
const (
	OptionsTypeVariantRed OptionsTypes = "Red"
	OptionsTypeVariantBanana OptionsTypes = "Banana"
	OptionsTypeVariantVermont OptionsTypes = "Vermont"
)
type Options struct{ 
	Type OptionsTypes `json:"type"`
	content interface{}
}

func (o *Options) UnmarshalJSON(data []byte) error {
	var enum struct {
		Tag    OptionsTypes   `json:"type"`
		Content json.RawMessage `json:"content"`
	}
	if err := json.Unmarshal(data, &enum); err != nil {
		return err
	}

	o.Type = enum.Tag
	switch o.Type {
	case OptionsTypeVariantRed:
		var res bool
		o.content = &res
	case OptionsTypeVariantBanana:
		var res string
		o.content = &res
	case OptionsTypeVariantVermont:
		var res Options
		o.content = &res

	}
	if err := json.Unmarshal(enum.Content, &o.content); err != nil {
		return err
	}

	return nil
}

func (o Options) MarshalJSON() ([]byte, error) {
    var enum struct {
		Tag    OptionsTypes   `json:"type"`
		Content interface{} `json:"content,omitempty"`
    }
    enum.Tag = o.Type
    enum.Content = o.content
    return json.Marshal(enum)
}

func (o Options) Red() bool {
	res, _ := o.content.(*bool)
	return *res
}
func (o Options) Banana() string {
	res, _ := o.content.(*string)
	return *res
}
func (o Options) Vermont() Options {
	res, _ := o.content.(*Options)
	return *res
}

func NewOptionsTypeVariantRed(content bool) Options {
    return Options{
        Type: OptionsTypeVariantRed,
        content: &content,
    }
}
func NewOptionsTypeVariantBanana(content string) Options {
    return Options{
        Type: OptionsTypeVariantBanana,
        content: &content,
    }
}
func NewOptionsTypeVariantVermont(content Options) Options {
    return Options{
        Type: OptionsTypeVariantVermont,
        content: &content,
    }
}

