package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class Card (
	val name: String
)

typealias Cards = List<Card>

@Serializable
data class Player (
	val name: String
)

typealias Players = List<Player>

typealias Pool = Cards

