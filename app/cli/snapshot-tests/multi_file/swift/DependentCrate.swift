import Foundation

public struct TypeBar: Codable {
	public let foo: TypeFoo
	public let s: String
	public let i: Int32

	public init(foo: TypeFoo, s: String, i: Int32) {
		self.foo = foo
		self.s = s
		self.i = i
	}
}
