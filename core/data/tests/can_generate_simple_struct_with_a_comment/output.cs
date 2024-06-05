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
	public string Name { get; set; }
	public ushort Age { get; set; }
	public string? Info { get; set; }
	public IEnumerable<string> Emails { get; set; }
	public Location Location { get; set; }
}

