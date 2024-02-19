#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class AddressDetails {
}

[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(FixedAddress), "FixedAddress")]
[JsonSubtypes.KnownSubType(typeof(NoFixedAddress), "NoFixedAddress")]
public abstract record Address 
{
	public record FixedAddress(AddressDetails Content) : Address();
	public record NoFixedAddress(): Address();
}


