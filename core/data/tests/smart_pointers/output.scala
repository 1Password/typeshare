package com.agilebits

package object onepassword {

type UByte = Byte
type UShort = Short
type UInt = Int
type ULong = Int

}
package onepassword {

// This is a comment.
case class ArcyColors (
	red: UByte,
	blue: String,
	green: Vector[String]
)

// This is a comment.
case class CellyColors (
	red: String,
	blue: Vector[String]
)

// This is a comment.
case class CowyColors (
	lifetime: String
)

// This is a comment.
case class LockyColors (
	red: String
)

// This is a comment.
case class MutexyColors (
	blue: Vector[String],
	green: String
)

// This is a comment.
case class RcyColors (
	red: String,
	blue: Vector[String],
	green: String
)

// This is a comment.
sealed trait BoxyColors {
	def serialName: String
}
object BoxyColors {
	case object Red extends BoxyColors {
		val serialName: String = "Red"
	}
	case object Blue extends BoxyColors {
		val serialName: String = "Blue"
	}
	case class Green(content: String) extends BoxyColors {
		val serialName: String = "Green"
	}
}

}
