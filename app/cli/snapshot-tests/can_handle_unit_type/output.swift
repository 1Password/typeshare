import Foundation

/// This struct has a unit field
public struct StructHasVoidType: Codable {
	public let thisIsAUnit: CodableVoid

	public init(thisIsAUnit: CodableVoid) {
		self.thisIsAUnit = thisIsAUnit
	}
}

/// This enum has a variant associated with unit data
public enum EnumHasVoidType: Codable {
	case hasAUnit(CodableVoid)

	enum CodingKeys: String, CodingKey, Codable {
		case hasAUnit
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .hasAUnit:
				if let content = try? container.decode(CodableVoid.self, forKey: .content) {
					self = .hasAUnit(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(EnumHasVoidType.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for EnumHasVoidType"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .hasAUnit(let content):
			try container.encode(CodingKeys.hasAUnit, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}

/// () isn't codable, so we use this instead to represent Rust's unit type
public struct CodableVoid: Codable, Equatable {}
