import Foundation

public struct A: Codable {
	public let field: UInt32

	public init(field: UInt32) {
		self.field = field
	}
}

public struct B: Codable {
	public let dependsOn: A

	public init(dependsOn: A) {
		self.dependsOn = dependsOn
	}
}

public struct C: Codable {
	public let dependsOn: B

	public init(dependsOn: B) {
		self.dependsOn = dependsOn
	}
}

public struct E: Codable {
	public let dependsOn: D

	public init(dependsOn: D) {
		self.dependsOn = dependsOn
	}
}

public struct D: Codable {
	public let dependsOn: C
	public let alsoDependsOn: E?

	public init(dependsOn: C, alsoDependsOn: E?) {
		self.dependsOn = dependsOn
		self.alsoDependsOn = alsoDependsOn
	}
}
