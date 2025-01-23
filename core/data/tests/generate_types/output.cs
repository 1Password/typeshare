#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class CustomType {
}

public class Types {
	[JsonProperty(Required = Required.Always)]
	public string S { get; set; }
	[JsonProperty(Required = Required.Always)]
	public string StaticS { get; set; }
	[JsonProperty(Required = Required.Always)]
	public short Int8 { get; set; }
	[JsonProperty(Required = Required.Always)]
	public float Float { get; set; }
	[JsonProperty(Required = Required.Always)]
	public double Double { get; set; }
	[JsonProperty(Required = Required.Always)]
	public IEnumerable<string> Array { get; set; }
	[JsonProperty(Required = Required.Always)]
	public string[] FixedLengthArray { get; set; }
	[JsonProperty(Required = Required.Always)]
	public IDictionary<string, int> Dictionary { get; set; }
	public IDictionary<string, int>? OptionalDictionary { get; set; }
	[JsonProperty(Required = Required.Always)]
	public CustomType CustomType { get; set; }
}

