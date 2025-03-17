package com.agilebits.onepassword

public record QualifiedTypes(
	String unqualified,
	String qualified,
	java.util.ArrayList<String> qualified_vec,
	java.util.HashMap<String, String> qualified_hashmap,
	String qualified_optional,
	java.util.HashMap<String, java.util.ArrayList<String>> qualfied_optional_hashmap_vec
) {}

