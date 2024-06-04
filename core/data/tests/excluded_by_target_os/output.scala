package com.agilebits

package onepassword {

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
}

sealed trait SomeEnum {
	def serialName: String
}
object SomeEnum {
}

}
