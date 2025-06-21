import Foundation

public struct PointerSizedType: Codable {
	public let unsigned: UInt64
	public let signed: Int64

	public init(unsigned: UInt64, signed: Int64) {
		self.unsigned = unsigned
		self.signed = signed
	}
}
