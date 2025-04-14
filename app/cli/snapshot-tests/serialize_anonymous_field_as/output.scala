package com.agilebits

package onepassword {

sealed trait SomeEnum {
	def serialName: String
}
object SomeEnum {
	// The associated String contains some opaque context
	case class Context(content: String) extends SomeEnum {
		val serialName: String = "Context"
	}
	case class Other(content: Int) extends SomeEnum {
		val serialName: String = "Other"
	}
}

}
