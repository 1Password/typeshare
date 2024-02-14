/** Generated type representing the anonymous struct variant `Us` of the `AutofilledBy` Rust enum */
public class AutofilledByUsInner {
	/** The UUID for the fill */
	public string Uuid { get; set; }
}

/** Generated type representing the anonymous struct variant `SomethingElse` of the `AutofilledBy` Rust enum */
public class AutofilledBySomethingElseInner {
	/** The UUID for the fill */
	public string Uuid { get; set; }
	/** Some other thing */
	public int Thing { get; set; }
}

/** Enum keeping track of who autofilled a field */
public record AutofilledBy 
{
	/** This field was autofilled by us */
	public record Us(AutofilledByUsInner Content): AutofilledBy();
	/** Something else autofilled this field */
	public record SomethingElse(AutofilledBySomethingElseInner Content): AutofilledBy();
}


/** Generated type representing the anonymous struct variant `AnonVariant` of the `EnumWithManyVariants` Rust enum */
public class EnumWithManyVariantsAnonVariantInner {
	public string Uuid { get; set; }
}

/** Generated type representing the anonymous struct variant `AnotherAnonVariant` of the `EnumWithManyVariants` Rust enum */
public class EnumWithManyVariantsAnotherAnonVariantInner {
	public string Uuid { get; set; }
	public int Thing { get; set; }
}

/** This is a comment (yareek sameek wuz here) */
public record EnumWithManyVariants 
{
	public record UnitVariant(): EnumWithManyVariants();
	public record TupleVariantString(string Content) : EnumWithManyVariants();
	public record AnonVariant(EnumWithManyVariantsAnonVariantInner Content): EnumWithManyVariants();
	public record TupleVariantInt(int Content) : EnumWithManyVariants();
	public record AnotherUnitVariant(): EnumWithManyVariants();
	public record AnotherAnonVariant(EnumWithManyVariantsAnotherAnonVariantInner Content): EnumWithManyVariants();
}


