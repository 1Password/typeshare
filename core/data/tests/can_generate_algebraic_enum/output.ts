/** Struct comment */
export interface ItemDetailsFieldValue {
}

/** Enum comment */
export type AdvancedColors = 
	/** This is a case comment */
	| { type: "String", content: string }
	| { type: "Number", content: number }
	| { type: "UnsignedNumber", content: number }
	| { type: "NumberArray", content: number[] }
	/** Comment on the last element */
	| { type: "ReallyCoolType", content: ItemDetailsFieldValue };

export type AdvancedColors2 = 
	/** This is a case comment */
	| { type: "string", content: string }
	| { type: "number", content: number }
	| { type: "number-array", content: number[] }
	/** Comment on the last element */
	| { type: "really-cool-type", content: ItemDetailsFieldValue };

