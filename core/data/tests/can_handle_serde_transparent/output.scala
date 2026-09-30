package com.agilebits

package object onepassword {

type Cards = Vector[Card]

type Players = Vector[Player]

type Pool = Cards

}
package onepassword {

case class Card (
	name: String
)

case class Player (
	name: String
)

}
