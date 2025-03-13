export interface CustomType {
}

export interface Types {
	s: string;
	static_s: string;
	int8: number;
	float: number;
	double: number;
	array: string[];
	fixed_length_array: [string, string, string, string];
	dictionary: Record<string, number>;
	optional_dictionary?: Record<string, number>;
	custom_type: CustomType;
}

