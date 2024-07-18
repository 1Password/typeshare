#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class ObjectNamedA {
	[JsonProperty(Required = Required.Always)]
	public string dependsOn { get; set; }
	[JsonProperty(Required = Required.Always)]
	public int age { get; set; }
	[JsonProperty(Required = Required.Always)]
	public string someStringValue { get; set; }
}

public enum DimensionFitValue
{
	[EnumMember(Value = "wrap-content")]
	WrapContent,

	[EnumMember(Value = "fit-height")]
	FitHeight,

}

[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(fixed-size), "fixed-size")]
[JsonSubtypes.KnownSubType(typeof(percentage), "percentage")]
[JsonSubtypes.KnownSubType(typeof(fit), "fit")]
public abstract record DimensionValue 
{
	public record FixedSize(float Value) : DimensionValue();
	public record Percentage(float Value) : DimensionValue();
	public record Fit(DimensionFitValue Value) : DimensionValue();
}


