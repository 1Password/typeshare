export interface OverrideStruct {
	readonly fieldToOverride: any | undefined;
}

export type OverrideEnum = 
	| { type: "UnitVariant", content?: undefined }
	| { type: "TupleVariant", content: string }
	| { type: "AnonymousStructVariant", content: {
	readonly fieldToOverride: any | undefined;
}};

