export type SomeEnum = 
	| { type: "A" }
	| { type: "B",
	field1: string;
}
	| { type: "C",
	field1: number;
	field2: number;
}
	| { type: "D",
	field3?: boolean;
};

