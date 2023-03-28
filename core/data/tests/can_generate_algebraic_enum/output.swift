import Foundation

/// Struct comment
public struct OPItemDetailsFieldValue: Codable {
	public init() {}
}

/// Enum comment
public enum OPAdvancedColors: Codable {
	/// This is a case comment
	case string(String)
	case number(Int32)
	case unsignedNumber(UInt32)
	case numberArray([Int32])
	/// Comment on the last element
	case reallyCoolType(OPItemDetailsFieldValue)

	enum CodingKeys: String, CodingKey, Codable {
		case string = "String",
			number = "Number",
			unsignedNumber = "UnsignedNumber",
			numberArray = "NumberArray",
			reallyCoolType = "ReallyCoolType"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .string:
				if let content = try? container.decode(String.self, forKey: .content) {
					self = .string(content)
					return
				}
			case .number:
				if let content = try? container.decode(Int32.self, forKey: .content) {
					self = .number(content)
					return
				}
			case .unsignedNumber:
				if let content = try? container.decode(UInt32.self, forKey: .content) {
					self = .unsignedNumber(content)
					return
				}
			case .numberArray:
				if let content = try? container.decode([Int32].self, forKey: .content) {
					self = .numberArray(content)
					return
				}
			case .reallyCoolType:
				if let content = try? container.decode(OPItemDetailsFieldValue.self, forKey: .content) {
					self = .reallyCoolType(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(OPAdvancedColors.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for OPAdvancedColors"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .string(let content):
			try container.encode(CodingKeys.string, forKey: .type)
			try container.encode(content, forKey: .content)
		case .number(let content):
			try container.encode(CodingKeys.number, forKey: .type)
			try container.encode(content, forKey: .content)
		case .unsignedNumber(let content):
			try container.encode(CodingKeys.unsignedNumber, forKey: .type)
			try container.encode(content, forKey: .content)
		case .numberArray(let content):
			try container.encode(CodingKeys.numberArray, forKey: .type)
			try container.encode(content, forKey: .content)
		case .reallyCoolType(let content):
			try container.encode(CodingKeys.reallyCoolType, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}

public enum OPAdvancedColors2: Codable {
	/// This is a case comment
	case string(String)
	case number(Int32)
	case numberArray([Int32])
	/// Comment on the last element
	case reallyCoolType(OPItemDetailsFieldValue)

	enum CodingKeys: String, CodingKey, Codable {
		case string,
			number,
			numberArray = "number-array",
			reallyCoolType = "really-cool-type"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .string:
				if let content = try? container.decode(String.self, forKey: .content) {
					self = .string(content)
					return
				}
			case .number:
				if let content = try? container.decode(Int32.self, forKey: .content) {
					self = .number(content)
					return
				}
			case .numberArray:
				if let content = try? container.decode([Int32].self, forKey: .content) {
					self = .numberArray(content)
					return
				}
			case .reallyCoolType:
				if let content = try? container.decode(OPItemDetailsFieldValue.self, forKey: .content) {
					self = .reallyCoolType(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(OPAdvancedColors2.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for OPAdvancedColors2"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .string(let content):
			try container.encode(CodingKeys.string, forKey: .type)
			try container.encode(content, forKey: .content)
		case .number(let content):
			try container.encode(CodingKeys.number, forKey: .type)
			try container.encode(content, forKey: .content)
		case .numberArray(let content):
			try container.encode(CodingKeys.numberArray, forKey: .type)
			try container.encode(content, forKey: .content)
		case .reallyCoolType(let content):
			try container.encode(CodingKeys.reallyCoolType, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}
