/** This is a Person struct with camelCase rename */
export interface Person {
	firstName: string;
	lastName: string;
	age: number;
	extraSpecialField1: number;
	extraSpecialField2?: string[];
}

/** This is a Person2 struct with UPPERCASE rename */
export interface Person2 {
	FIRST_NAME: string;
	LAST_NAME: string;
	AGE: number;
}

