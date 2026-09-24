type customType = {
}

type types = {
	s: string,
	static_s: string,
	int8: int,
	float: float,
	double: float,
	array: array<string>,
	fixed_length_array: array<string> /* length: 4 */,
	dictionary: Dict.t<int>,
	optional_dictionary?: Dict.t<int>,
	custom_type: customType,
}

