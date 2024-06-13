#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

/** This is a Person struct with camelCase rename */
public class Person {
	[JsonProperty(Required = Required.Always)]
	public string firstName { get; set; }
	[JsonProperty(Required = Required.Always)]
	public string lastName { get; set; }
	[JsonProperty(Required = Required.Always)]
	public ushort age { get; set; }
	[JsonProperty(Required = Required.Always)]
	public int extraSpecialField1 { get; set; }
	public IEnumerable<string>? extraSpecialField2 { get; set; }
}

/** This is a Person2 struct with UPPERCASE rename */
public class Person2 {
	[JsonProperty(Required = Required.Always)]
	public string FIRST_NAME { get; set; }
	[JsonProperty(Required = Required.Always)]
	public string LAST_NAME { get; set; }
	[JsonProperty(Required = Required.Always)]
	public ushort AGE { get; set; }
}

