export type SomeEnum = 
	/** The associated String contains some opaque context */
	| { type: "Context", content: string }
	| { type: "Other", content: number };

