/** This struct has a unit field */
export interface StructHasVoidType {
	thisIsAUnit: undefined;
}

/** This enum has a variant associated with unit data */
export type EnumHasVoidType = 
	| { type: "hasAUnit", content: undefined };

