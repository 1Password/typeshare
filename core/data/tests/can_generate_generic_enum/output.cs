#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(VariantA), "VariantA")]
[JsonSubtypes.KnownSubType(typeof(VariantB), "VariantB")]
public abstract record GenericEnum<TA, TB> 
{
	public record VariantA(TA Content) : GenericEnum<TA, TB>();
	public record VariantB(TB Content) : GenericEnum<TA, TB>();
}


public class StructUsingGenericEnum {
	public GenericEnum<string, short> EnumField { get; set; }
}

[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(VariantC), "VariantC")]
[JsonSubtypes.KnownSubType(typeof(VariantD), "VariantD")]
[JsonSubtypes.KnownSubType(typeof(VariantE), "VariantE")]
public abstract record GenericEnumUsingGenericEnum<T> 
{
	public record VariantC(GenericEnum<T, T> Content) : GenericEnumUsingGenericEnum<T>();
	public record VariantD(GenericEnum<string, IDictionary<string, T>> Content) : GenericEnumUsingGenericEnum<T>();
	public record VariantE(GenericEnum<string, uint> Content) : GenericEnumUsingGenericEnum<T>();
}


/** Generated type representing the anonymous struct variant `VariantF` of the `GenericEnumsUsingStructVariants` Rust enum */
public class GenericEnumsUsingStructVariantsVariantFInner<T> {
	public T Action { get; set; }
}

/** Generated type representing the anonymous struct variant `VariantG` of the `GenericEnumsUsingStructVariants` Rust enum */
public class GenericEnumsUsingStructVariantsVariantGInner<T, TU> {
	public T Action { get; set; }
	public TU Response { get; set; }
}

/** Generated type representing the anonymous struct variant `VariantH` of the `GenericEnumsUsingStructVariants` Rust enum */
public class GenericEnumsUsingStructVariantsVariantHInner {
	public int NonGeneric { get; set; }
}

/** Generated type representing the anonymous struct variant `VariantI` of the `GenericEnumsUsingStructVariants` Rust enum */
public class GenericEnumsUsingStructVariantsVariantIInner<T, TU> {
	public IEnumerable<T> Vec { get; set; }
	public MyType<T, TU> Action { get; set; }
}

[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(VariantF), "VariantF")]
[JsonSubtypes.KnownSubType(typeof(VariantG), "VariantG")]
[JsonSubtypes.KnownSubType(typeof(VariantH), "VariantH")]
[JsonSubtypes.KnownSubType(typeof(VariantI), "VariantI")]
public abstract record GenericEnumsUsingStructVariants<T, TU> 
{
	public record VariantF(GenericEnumsUsingStructVariantsVariantFInner<T> Content): GenericEnumsUsingStructVariants<T, TU>();
	public record VariantG(GenericEnumsUsingStructVariantsVariantGInner<T, TU> Content): GenericEnumsUsingStructVariants<T, TU>();
	public record VariantH(GenericEnumsUsingStructVariantsVariantHInner Content): GenericEnumsUsingStructVariants<T, TU>();
	public record VariantI(GenericEnumsUsingStructVariantsVariantIInner<T, TU> Content): GenericEnumsUsingStructVariants<T, TU>();
}


