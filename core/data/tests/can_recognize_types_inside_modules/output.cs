#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class A {
	public uint Field { get; set; }
}

public class ABC {
	public uint Field { get; set; }
}

public class AB {
	public uint Field { get; set; }
}

public class OutsideOfModules {
	public uint Field { get; set; }
}

