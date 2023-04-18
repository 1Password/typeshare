export interface AddressDetails {
}

export type Address = 
	| { type: "FixedAddress", content: AddressDetails }
	| { type: "NoFixedAddress", content?: undefined };

