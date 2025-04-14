package com.agilebits

package onepassword {

// A struct with no target_os. Should be generated when
// we use --target-os.
class AlwaysAccept extends Serializable

case class DefinedTwice (
	field1: String
)

class Excluded extends Serializable

class ManyStruct extends Serializable

class MultipleTargets extends Serializable

class NestedNotTarget1 extends Serializable

class OtherExcluded extends Serializable

sealed trait AlwaysAcceptEnum {
	def serialName: String
}
object AlwaysAcceptEnum {
	case object Variant1 extends AlwaysAcceptEnum {
		val serialName: String = "Variant1"
	}
	case object Variant2 extends AlwaysAcceptEnum {
		val serialName: String = "Variant2"
	}
}

sealed trait SomeEnum {
	def serialName: String
}
object SomeEnum {
}

// Generated type representing the anonymous struct variant `Variant7` of the `TestEnum` Rust enum
case class TestEnumVariant7Inner (
	field1: String
)

// Generated type representing the anonymous struct variant `Variant9` of the `TestEnum` Rust enum
case class TestEnumVariant9Inner (
	field2: String
)

sealed trait TestEnum {
	def serialName: String
}
object TestEnum {
	case object Variant1 extends TestEnum {
		val serialName: String = "Variant1"
	}
	case object Variant5 extends TestEnum {
		val serialName: String = "Variant5"
	}
	case class Variant7(content: TestEnumVariant7Inner) extends TestEnum {
		val serialName: String = "Variant7"
	}
	case object Variant8 extends TestEnum {
		val serialName: String = "Variant8"
	}
	case class Variant9(content: TestEnumVariant9Inner) extends TestEnum {
		val serialName: String = "Variant9"
	}
}

}
