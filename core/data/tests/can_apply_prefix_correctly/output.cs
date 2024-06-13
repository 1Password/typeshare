#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class ItemDetailsFieldValue {
	[JsonProperty(Required = Required.Always)]
	public string Hello { get; set; }
}

[JsonConverter(typeof(JsonSubtypes), "t")]
[JsonSubtypes.KnownSubType(typeof(String), "String")]
[JsonSubtypes.KnownSubType(typeof(Number), "Number")]
[JsonSubtypes.KnownSubType(typeof(NumberArray), "NumberArray")]
[JsonSubtypes.KnownSubType(typeof(ReallyCoolType), "ReallyCoolType")]
[JsonSubtypes.KnownSubType(typeof(ArrayReallyCoolType), "ArrayReallyCoolType")]
[JsonSubtypes.KnownSubType(typeof(DictionaryReallyCoolType), "DictionaryReallyCoolType")]
public abstract record AdvancedColors 
{
	public record String(string C) : AdvancedColors();
	public record Number(int C) : AdvancedColors();
	public record NumberArray(IEnumerable<int> C) : AdvancedColors();
	public record ReallyCoolType(ItemDetailsFieldValue C) : AdvancedColors();
	public record ArrayReallyCoolType(IEnumerable<ItemDetailsFieldValue> C) : AdvancedColors();
	public record DictionaryReallyCoolType(IDictionary<string, ItemDetailsFieldValue> C) : AdvancedColors();
}


