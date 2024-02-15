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

public class CustomType {
}

public class Types {
	public string S { get; set; }
	public string StaticS { get; set; }
	public short Int8 { get; set; }
	public float Float { get; set; }
	public double Double { get; set; }
	public IEnumerable<string> Array { get; set; }
	public string[] FixedLengthArray { get; set; }
	public IDictionary<string, int> Dictionary { get; set; }
	public IDictionary<string, int>? OptionalDictionary { get; set; }
	public CustomType CustomType { get; set; }
}

