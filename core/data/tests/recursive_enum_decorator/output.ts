export type Options = 
	| { type: "red", content: boolean }
	| { type: "banana", content: string }
	| { type: "vermont", content: Options };

export type MoreOptions = 
	| { type: "news", content: boolean }
	| { type: "exactly", content: {
	config: string;
}}
	| { type: "built", content: {
	top: MoreOptions;
}};

