export type GenericEnum<A, B> = 
	| { type: "VariantA", content: A }
	| { type: "VariantB", content: B };

export interface StructUsingGenericEnum {
	enum_field: GenericEnum<string, number>;
}

export type GenericEnumUsingGenericEnum<T> = 
	| { type: "VariantC", content: GenericEnum<T, T> }
	| { type: "VariantD", content: GenericEnum<string, Record<string, T>> }
	| { type: "VariantE", content: GenericEnum<string, number> };

export type GenericEnumsUsingStructVariants<T, U> = 
	| { type: "VariantF", content: {
	action: T;
}}
	| { type: "VariantG", content: {
	action: T;
	response: U;
}}
	| { type: "VariantH", content: {
	non_generic: number;
}}
	| { type: "VariantI", content: {
	vec: T[];
	action: MyType<T, U>;
}};

