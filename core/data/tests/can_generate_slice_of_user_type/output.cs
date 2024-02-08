#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class Video {
	public IEnumerable<Tag> Tags { get; set; }
}

