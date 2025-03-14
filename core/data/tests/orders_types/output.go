package proto

import "encoding/json"

type A struct {
	Field uint32 `json:"field"`
}
type B struct {
	DependsOn A `json:"dependsOn"`
}
type C struct {
	DependsOn B `json:"dependsOn"`
}
type E struct {
	DependsOn D `json:"dependsOn"`
}
type D struct {
	DependsOn C `json:"dependsOn"`
	AlsoDependsOn *E `json:"alsoDependsOn,omitempty"`
}
