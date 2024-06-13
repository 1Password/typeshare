#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class QualifiedTypes {
	[JsonProperty(Required = Required.Always)]
	public string Unqualified { get; set; }
	[JsonProperty(Required = Required.Always)]
	public string Qualified { get; set; }
	[JsonProperty(Required = Required.Always)]
	public IEnumerable<string> QualifiedVec { get; set; }
	[JsonProperty(Required = Required.Always)]
	public IDictionary<string, string> QualifiedHashmap { get; set; }
	public string? QualifiedOptional { get; set; }
	public IDictionary<string, IEnumerable<string>>? QualfiedOptionalHashmapVec { get; set; }
}

