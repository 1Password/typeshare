import Foundation

public struct Test: Codable {
	public let field1: String
	public let field2: UInt32

	public init(field1: String, field2: UInt32) {
		self.field1 = field1
		self.field2 = field2
	}
}
