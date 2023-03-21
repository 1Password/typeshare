export interface QualifiedTypes {
	unqualified: string;
	qualified: string;
	qualified_vec: string[];
	qualified_hashmap: Record<string, string>;
	qualified_optional?: string;
	qualfied_optional_hashmap_vec?: Record<string, string[]>;
}

