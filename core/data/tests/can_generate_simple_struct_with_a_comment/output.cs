#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class Location {
}

/** This is a comment. */
public class Person {
	/** This is another comment */
	[JsonProperty(Required = Required.Always)]
	public string Name { get; set; }
	[JsonProperty(Required = Required.Always)]
	public ushort Age { get; set; }
	public string? Info { get; set; }
	[JsonProperty(Required = Required.Always)]
	public IEnumerable<string> Emails { get; set; }
	[JsonProperty(Required = Required.Always)]
	public Location Location { get; set; }
}

