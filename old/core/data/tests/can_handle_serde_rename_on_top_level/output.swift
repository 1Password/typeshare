import Foundation

public struct OPOtherType: Codable {
	public init() {}
}

/// This is a comment.
public struct OPPersonTwo: Codable {
	public let name: String
	public let age: UInt8
	public let extraSpecialFieldOne: Int32
	public let extraSpecialFieldTwo: [String]?
	public let nonStandardDataType: OPOtherType
	public let nonStandardDataTypeInArray: [OPOtherType]?

	public init(name: String, age: UInt8, extraSpecialFieldOne: Int32, extraSpecialFieldTwo: [String]?, nonStandardDataType: OPOtherType, nonStandardDataTypeInArray: [OPOtherType]?) {
		self.name = name
		self.age = age
		self.extraSpecialFieldOne = extraSpecialFieldOne
		self.extraSpecialFieldTwo = extraSpecialFieldTwo
		self.nonStandardDataType = nonStandardDataType
		self.nonStandardDataTypeInArray = nonStandardDataTypeInArray
	}
}
