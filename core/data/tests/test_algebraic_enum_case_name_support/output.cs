#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class ItemDetailsFieldValue {
}

[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(String), "string")]
[JsonSubtypes.KnownSubType(typeof(Number), "number")]
[JsonSubtypes.KnownSubType(typeof(NumberArray), "number-array")]
[JsonSubtypes.KnownSubType(typeof(ReallyCoolType), "reallyCoolType")]
public abstract record AdvancedColors 
{
	public record String(string Content) : AdvancedColors();
	public record Number(int Content) : AdvancedColors();
	public record NumberArray(IEnumerable<int> Content) : AdvancedColors();
	public record ReallyCoolType(ItemDetailsFieldValue Content) : AdvancedColors();
}


