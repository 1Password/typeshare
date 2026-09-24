/** This is a Person struct with camelCase rename */
type person = {
	firstName: string,
	lastName: string,
	age: int,
	extraSpecialField1: int,
	extraSpecialField2?: array<string>,
}

/** This is a Person2 struct with UPPERCASE rename */
type person2 = {
	FIRST_NAME: string,
	LAST_NAME: string,
	AGE: int,
}

