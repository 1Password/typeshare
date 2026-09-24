type otherType = {
}

/** This is a comment. */
type person = {
	name: string,
	age: int,
	extraSpecialFieldOne: int,
	extraSpecialFieldTwo?: array<string>,
	nonStandardDataType: otherType,
	nonStandardDataTypeInArray?: array<otherType>,
}

