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

