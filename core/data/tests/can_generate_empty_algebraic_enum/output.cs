public class AddressDetails {
}

public record Address 
{
	public record FixedAddress(AddressDetails Content) : Address();
	public record NoFixedAddress(): Address();
}


