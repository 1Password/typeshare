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

/** This is a comment. */
public class Foo {
	public short A { get; set; }
	public short B { get; set; }
	public int C { get; set; }
	public ushort E { get; set; }
	public ushort F { get; set; }
	public uint G { get; set; }
}

