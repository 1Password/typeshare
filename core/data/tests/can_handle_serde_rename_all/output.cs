#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

/** This is a Person struct with camelCase rename */
public class Person {
	public string firstName { get; set; }
	public string lastName { get; set; }
	public ushort age { get; set; }
	public int extraSpecialField1 { get; set; }
	public IEnumerable<string>? extraSpecialField2 { get; set; }
}

/** This is a Person2 struct with UPPERCASE rename */
public class Person2 {
	public string FIRST_NAME { get; set; }
	public string LAST_NAME { get; set; }
	public ushort AGE { get; set; }
}

