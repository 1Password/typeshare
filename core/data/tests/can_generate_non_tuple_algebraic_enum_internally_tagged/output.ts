export type NonTupleAlgebraicEnum = 
	| { type: "VariantA"; 
	foo: number;
}
	| { type: "VariantB"; 
	foo: number;
	bar: string;
}
	| { type: "VariantC" }
	| { type: "VariantD" };

