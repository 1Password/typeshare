package com.agilebits

package onepassword {

// This is a comment.
sealed trait Colors {
	def serialName: String
}
object Colors {
	case object Green extends Colors {
		val serialName: String = "Green\""
	}
}

}
