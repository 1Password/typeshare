#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class ItemDetailsFieldValue {
}

[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(String), "String")]
[JsonSubtypes.KnownSubType(typeof(Number), "Number")]
[JsonSubtypes.KnownSubType(typeof(NumberArray), "NumberArray")]
[JsonSubtypes.KnownSubType(typeof(ReallyCoolType), "ReallyCoolType")]
public abstract record AdvancedColors 
{
	public record String(string Content) : AdvancedColors();
	public record Number(int Content) : AdvancedColors();
	public record NumberArray(IEnumerable<int> Content) : AdvancedColors();
	public record ReallyCoolType(ItemDetailsFieldValue Content) : AdvancedColors();
}


