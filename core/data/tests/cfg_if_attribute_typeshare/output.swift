import Foundation

public typealias Bytes = [UInt8]

/// Example of a type that is conditionally typeshared
/// based on a feature "typeshare-support". This does not
/// conditionally typeshare but allows a conditionally
/// typeshared type to generate typeshare types when behind
/// a `cfg_attr` condition.
public struct TestStruct1: Codable {
	public let field: String

	public init(field: String) {
		self.field = field
	}
}

public struct TestStruct2<R: Codable & Equatable & Hashable>: Codable, Equatable, Hashable {
	public let field1: String
	public let field2: R

	public init(field1: String, field2: R) {
		self.field1 = field1
		self.field2 = field2
	}
}

public struct TestStruct3: Codable {
	public let field_1: String

	public init(field_1: String) {
		self.field_1 = field_1
	}
}
