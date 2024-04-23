package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class Video (
	val tags: List<Tag>
)

