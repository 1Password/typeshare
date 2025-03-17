package com.agilebits.onepassword

/// This is a Person struct with camelCase rename
public record Person(
	String firstName,
	String lastName,
	short age,
	int extraSpecialField1,
	java.util.ArrayList<String> extraSpecialField2
) {}

/// This is a Person2 struct with UPPERCASE rename
public record Person2(
	String FIRST_NAME,
	String LAST_NAME,
	short AGE
) {}

