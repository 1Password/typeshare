public record GenericEnum<TA, TB> 
{
	public record VariantA(TA Content) : GenericEnum<TA, TB>();
	public record VariantB(TB Content) : GenericEnum<TA, TB>();
}


public class StructUsingGenericEnum {
	public GenericEnum<string, short> EnumField { get; set; }
}

public record GenericEnumUsingGenericEnum<T> 
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

public record GenericEnumsUsingStructVariants<T, TU> 
{
	public record VariantF(GenericEnumsUsingStructVariantsVariantFInner<T> Content): GenericEnumsUsingStructVariants<T, TU>();
	public record VariantG(GenericEnumsUsingStructVariantsVariantGInner<T, TU> Content): GenericEnumsUsingStructVariants<T, TU>();
	public record VariantH(GenericEnumsUsingStructVariantsVariantHInner Content): GenericEnumsUsingStructVariants<T, TU>();
	public record VariantI(GenericEnumsUsingStructVariantsVariantIInner<T, TU> Content): GenericEnumsUsingStructVariants<T, TU>();
}


