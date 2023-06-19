import Foundation

public struct MyType: Codable {
	public let field: Unicode.Scalar

	public init(field: Unicode.Scalar) {
		self.field = field
	}
}
