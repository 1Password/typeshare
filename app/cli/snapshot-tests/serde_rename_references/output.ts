export type AliasTest = SomethingFoo[];

export interface Test {
	field1: SomethingFoo;
	field2?: SomethingFoo;
}

export enum SomethingFoo {
	A = "A",
}

export type Parent = 
	| { type: "B", value: SomethingFoo };

