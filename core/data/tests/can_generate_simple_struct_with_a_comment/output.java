package com.agilebits.onepassword

public record Location() {}

/// This is a comment.
public record Person(
	/// This is another comment
	String name,
	short age,
	String info,
	java.util.ArrayList<String> emails,
	Location location
) {}

