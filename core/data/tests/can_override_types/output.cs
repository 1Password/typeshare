#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class OverrideStruct {
	public char fieldToOverride { get; set; }
}

/** Generated type representing the anonymous struct variant `AnonymousStructVariant` of the `OverrideEnum` Rust enum */
public class OverrideEnumAnonymousStructVariantInner {
	public char fieldToOverride { get; set; }
}

[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(UnitVariant), "UnitVariant")]
[JsonSubtypes.KnownSubType(typeof(TupleVariant), "TupleVariant")]
[JsonSubtypes.KnownSubType(typeof(AnonymousStructVariant), "AnonymousStructVariant")]
public abstract record OverrideEnum 
{
	public record UnitVariant(): OverrideEnum();
	public record TupleVariant(string Content) : OverrideEnum();
	public record AnonymousStructVariant(OverrideEnumAnonymousStructVariantInner Content): OverrideEnum();
}


