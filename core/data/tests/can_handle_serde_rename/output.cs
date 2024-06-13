#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class OtherType {
}

/** This is a comment. */
public class Person {
	[JsonProperty(Required = Required.Always)]
	public string name { get; set; }
	[JsonProperty(Required = Required.Always)]
	public ushort age { get; set; }
	[JsonProperty(Required = Required.Always)]
	public int extraSpecialFieldOne { get; set; }
	public IEnumerable<string>? extraSpecialFieldTwo { get; set; }
	[JsonProperty(Required = Required.Always)]
	public OtherType nonStandardDataType { get; set; }
	public IEnumerable<OtherType>? nonStandardDataTypeInArray { get; set; }
}

