export interface OtherType {
}

/** This is a comment. */
export interface Person {
	name: string;
	age: number;
	extraSpecialFieldOne: number;
	extraSpecialFieldTwo?: string[];
	nonStandardDataType: OtherType;
	nonStandardDataTypeInArray?: OtherType[];
}

