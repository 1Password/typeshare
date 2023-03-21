package proto

import "encoding/json"

type QualifiedTypes struct {
	Unqualified string `json:"unqualified"`
	Qualified string `json:"qualified"`
	QualifiedVec []string `json:"qualified_vec"`
	QualifiedHashmap map[string]string `json:"qualified_hashmap"`
	QualifiedOptional *string `json:"qualified_optional,omitempty"`
	QualfiedOptionalHashmapVec *map[string][]string `json:"qualfied_optional_hashmap_vec,omitempty"`
}
