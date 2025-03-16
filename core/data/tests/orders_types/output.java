package com.agilebits.onepassword

public record A(
	long field
) {}

public record B(
	A dependsOn
) {}

public record C(
	B dependsOn
) {}

public record E(
	D dependsOn
) {}

public record D(
	C dependsOn,
	E alsoDependsOn
) {}

