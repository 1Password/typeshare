import Foundation

public struct TypeFoo: Codable {
	public let s: String
	public let i: Int32

	public init(s: String, i: Int32) {
		self.s = s
		self.i = i
	}
}
