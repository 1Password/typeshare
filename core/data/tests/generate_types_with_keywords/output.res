type catch = {
	default: string,
	case: string,
}

@tag("type")
type \"switch" = 
	| @as("default") Default({ content: catch })

type throws = 
	| @as("case") Case
	| @as("default") Default


