export interface ItemDetailsFieldValue {
}

export type AdvancedColors = 
	| { type: "string", content: string }
	| { type: "number", content: number }
	| { type: "number-array", content: number[] }
	| { type: "reallyCoolType", content: ItemDetailsFieldValue };

