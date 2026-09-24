type otherType = {
}

/** This is a comment. */
type personTwo = {
	name: string,
	age: int,
	extraSpecialFieldOne: int,
	extraSpecialFieldTwo?: array<string>,
	nonStandardDataType: otherType,
	nonStandardDataTypeInArray?: array<otherType>,
}

