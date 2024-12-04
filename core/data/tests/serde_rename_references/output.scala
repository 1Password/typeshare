package com.agilebits

package object onepassword {

type AliasTest = Vector[SomethingFoo]

}
package onepassword {

case class Test (
	field1: SomethingFoo,
	field2: Option[SomethingFoo] = None
)

sealed trait SomethingFoo {
	def serialName: String
}
object SomethingFoo {
	case object A extends SomethingFoo {
		val serialName: String = "A"
	}
}

sealed trait Parent {
	def serialName: String
}
object Parent {
	case class B(value: SomethingFoo) extends Parent {
		val serialName: String = "B"
	}
}

}
