#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

namespace Company.Domain.Models;

/** Struct comment */
public class ItemDetailsFieldValue {
}

/** Enum comment */
[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(String), "String")]
[JsonSubtypes.KnownSubType(typeof(Number), "Number")]
[JsonSubtypes.KnownSubType(typeof(UnsignedNumber), "UnsignedNumber")]
[JsonSubtypes.KnownSubType(typeof(NumberArray), "NumberArray")]
[JsonSubtypes.KnownSubType(typeof(ReallyCoolType), "ReallyCoolType")]
public abstract record AdvancedColors 
{
	/** This is a case comment */
	public record String(string Content) : AdvancedColors();
	public record Number(int Content) : AdvancedColors();
	public record UnsignedNumber(uint Content) : AdvancedColors();
	public record NumberArray(IEnumerable<int> Content) : AdvancedColors();
	/** Comment on the last element */
	public record ReallyCoolType(ItemDetailsFieldValue Content) : AdvancedColors();
}


[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(String), "string")]
[JsonSubtypes.KnownSubType(typeof(Number), "number")]
[JsonSubtypes.KnownSubType(typeof(NumberArray), "number-array")]
[JsonSubtypes.KnownSubType(typeof(ReallyCoolType), "really-cool-type")]
public abstract record AdvancedColors2 
{
	/** This is a case comment */
	public record String(string Content) : AdvancedColors2();
	public record Number(int Content) : AdvancedColors2();
	public record NumberArray(IEnumerable<int> Content) : AdvancedColors2();
	/** Comment on the last element */
	public record ReallyCoolType(ItemDetailsFieldValue Content) : AdvancedColors2();
}


