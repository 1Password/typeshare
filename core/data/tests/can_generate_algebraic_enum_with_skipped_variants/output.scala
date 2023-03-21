package com.agilebits

package onepassword {

sealed trait SomeEnum {
	def serialName: String
}
object SomeEnum {
	case object A extends SomeEnum {
		val serialName: String = "A"
	}
	case class C(content: Int) extends SomeEnum {
		val serialName: String = "C"
	}
}

}
