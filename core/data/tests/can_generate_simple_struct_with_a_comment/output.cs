#nullable enable

using System;
using System.Reflection;
using System.Collections.Generic;

class EnumLabelAttribute : Attribute
{
    public string Label { get; }

    public EnumLabelAttribute(string label)
    {
        Label = label;
    }
}

public static class EnumExtensions
{
    public static string Label<T>(this T value)
        where T : Enum
    {
        var fieldName = value.ToString();
        var field = typeof(T).GetField(fieldName, BindingFlags.Public | BindingFlags.Static);
        return field?.GetCustomAttribute<EnumLabelAttribute>()?.Label ?? fieldName;
    }
}

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

