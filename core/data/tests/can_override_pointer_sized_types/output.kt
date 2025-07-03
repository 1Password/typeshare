package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class PointerSizedType (
	val unsigned: ULong,
	val signed: Long
)

