package com.agilebits

package onepassword {

// Generated type representing the anonymous struct variant `Us` of the `AutofilledBy` Rust enum
case class AutofilledByUsInner (
	// The UUID for the fill
	uuid: String
)

// Generated type representing the anonymous struct variant `SomethingElse` of the `AutofilledBy` Rust enum
case class AutofilledBySomethingElseInner (
	// The UUID for the fill
	uuid: String
)

// Enum keeping track of who autofilled a field
sealed trait AutofilledBy {
	def serialName: String
}
object AutofilledBy {
	// This field was autofilled by us
	case class Us(content: AutofilledByUsInner) extends AutofilledBy {
		val serialName: String = "Us"
	}
	// Something else autofilled this field
	case class SomethingElse(content: AutofilledBySomethingElseInner) extends AutofilledBy {
		val serialName: String = "SomethingElse"
	}
}

}
