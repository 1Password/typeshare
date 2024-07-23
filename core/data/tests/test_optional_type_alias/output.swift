import Foundation

public typealias OptionalU16 = UInt16?

public typealias OptionalU32 = UInt32?

public struct FooBar: Codable {
	public let foo: OptionalU32
	public let bar: OptionalU16

	public init(foo: OptionalU32, bar: OptionalU16) {
		self.foo = foo
		self.bar = bar
	}
}
