#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

/** This is a comment. */
public class Foo {
	[JsonProperty(Required = Required.Always)]
	public short A { get; set; }
	[JsonProperty(Required = Required.Always)]
	public short B { get; set; }
	[JsonProperty(Required = Required.Always)]
	public int C { get; set; }
	[JsonProperty(Required = Required.Always)]
	public ushort E { get; set; }
	[JsonProperty(Required = Required.Always)]
	public ushort F { get; set; }
	[JsonProperty(Required = Required.Always)]
	public uint G { get; set; }
}

