package com.agilebits

package onepassword {

class MultipleTargets extends Serializable

sealed trait SomeEnum {
	def serialName: String
}
object SomeEnum {
}

// Generated type representing the anonymous struct variant `Variant7` of the `TestEnum` Rust enum
class TestEnumVariant7Inner extends Serializable

sealed trait TestEnum {
	def serialName: String
}
object TestEnum {
	case object Variant5 extends TestEnum {
		val serialName: String = "Variant5"
	}
	case class Variant7(content: TestEnumVariant7Inner) extends TestEnum {
		val serialName: String = "Variant7"
	}
	case object Variant8 extends TestEnum {
		val serialName: String = "Variant8"
	}
}

}
