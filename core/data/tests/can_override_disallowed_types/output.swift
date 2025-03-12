import Foundation

public struct DisallowedType: Codable {
	public let disallowed_type: UInt64
	public let another_disallowed_type: Int64
	public let disallowed_type_serde_with: UInt64

	public init(disallowed_type: UInt64, another_disallowed_type: Int64, disallowed_type_serde_with: UInt64) {
		self.disallowed_type = disallowed_type
		self.another_disallowed_type = another_disallowed_type
		self.disallowed_type_serde_with = disallowed_type_serde_with
	}
}
