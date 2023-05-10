import Foundation

public struct GenericType<T: Codable & Identifiable & Sendable>: Codable {
	public let field: T

	public init(field: T) {
		self.field = field
	}
}
