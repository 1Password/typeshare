#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

/** Generated type representing the anonymous struct variant `Us` of the `AutofilledBy` Rust enum */
public class AutofilledByUsInner {
	/** The UUID for the fill */
	[JsonProperty(Required = Required.Always)]
	public string Uuid { get; set; }
}

/** Generated type representing the anonymous struct variant `SomethingElse` of the `AutofilledBy` Rust enum */
public class AutofilledBySomethingElseInner {
	/** The UUID for the fill */
	[JsonProperty(Required = Required.Always)]
	public string Uuid { get; set; }
	/** Some other thing */
	[JsonProperty(Required = Required.Always)]
	public int Thing { get; set; }
}

/** Enum keeping track of who autofilled a field */
[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(Us), "Us")]
[JsonSubtypes.KnownSubType(typeof(SomethingElse), "SomethingElse")]
public abstract record AutofilledBy 
{
	/** This field was autofilled by us */
	public record Us(AutofilledByUsInner Content): AutofilledBy();
	/** Something else autofilled this field */
	public record SomethingElse(AutofilledBySomethingElseInner Content): AutofilledBy();
}


/** Generated type representing the anonymous struct variant `AnonVariant` of the `EnumWithManyVariants` Rust enum */
public class EnumWithManyVariantsAnonVariantInner {
	[JsonProperty(Required = Required.Always)]
	public string Uuid { get; set; }
}

/** Generated type representing the anonymous struct variant `AnotherAnonVariant` of the `EnumWithManyVariants` Rust enum */
public class EnumWithManyVariantsAnotherAnonVariantInner {
	[JsonProperty(Required = Required.Always)]
	public string Uuid { get; set; }
	[JsonProperty(Required = Required.Always)]
	public int Thing { get; set; }
}

/** This is a comment (yareek sameek wuz here) */
[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(UnitVariant), "UnitVariant")]
[JsonSubtypes.KnownSubType(typeof(TupleVariantString), "TupleVariantString")]
[JsonSubtypes.KnownSubType(typeof(AnonVariant), "AnonVariant")]
[JsonSubtypes.KnownSubType(typeof(TupleVariantInt), "TupleVariantInt")]
[JsonSubtypes.KnownSubType(typeof(AnotherUnitVariant), "AnotherUnitVariant")]
[JsonSubtypes.KnownSubType(typeof(AnotherAnonVariant), "AnotherAnonVariant")]
public abstract record EnumWithManyVariants 
{
	public record UnitVariant(): EnumWithManyVariants();
	public record TupleVariantString(string Content) : EnumWithManyVariants();
	public record AnonVariant(EnumWithManyVariantsAnonVariantInner Content): EnumWithManyVariants();
	public record TupleVariantInt(int Content) : EnumWithManyVariants();
	public record AnotherUnitVariant(): EnumWithManyVariants();
	public record AnotherAnonVariant(EnumWithManyVariantsAnotherAnonVariantInner Content): EnumWithManyVariants();
}


