import Foundation

public struct A: Codable {
	public let field: UInt32

	public init(field: UInt32) {
		self.field = field
	}
}

public struct AB: Codable {
	public let field: UInt32

	public init(field: UInt32) {
		self.field = field
	}
}

public struct ABC: Codable {
	public let field: UInt32

	public init(field: UInt32) {
		self.field = field
	}
}

public struct OutsideOfModules: Codable {
	public let field: UInt32

	public init(field: UInt32) {
		self.field = field
	}
}
