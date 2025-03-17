package com.agilebits.onepassword

public record CustomType() {}

public record Types(
	String s,
	String static_s,
	byte int8,
	float _float,
	double _double,
	java.util.ArrayList<String> array,
	String[] fixed_length_array,
	java.util.HashMap<String, int> dictionary,
	java.util.HashMap<String, int> optional_dictionary,
	CustomType custom_type
) {}

