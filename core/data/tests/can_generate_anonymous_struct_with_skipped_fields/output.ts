/** Enum keeping track of who autofilled a field */
export type AutofilledBy = 
	/** This field was autofilled by us */
	| { type: "Us", content: {
	/** The UUID for the fill */
	uuid: string;
}}
	/** Something else autofilled this field */
	| { type: "SomethingElse", content: {
	/** The UUID for the fill */
	uuid: string;
}};

