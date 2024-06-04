import Foundation

public struct CoreGenericStruct<A: Codable, B: Codable>: Codable {
	public let field_a: A
	public let field_b: [B]

	public init(field_a: A, field_b: [B]) {
		self.field_a = field_a
		self.field_b = field_b
	}
}

public struct CoreGenericStructUsingGenericStruct<T: Codable>: Codable {
	public let struct_field: CoreGenericStruct<String, T>
	public let second_struct_field: CoreGenericStruct<T, String>
	public let third_struct_field: CoreGenericStruct<T, [T]>

	public init(struct_field: CoreGenericStruct<String, T>, second_struct_field: CoreGenericStruct<T, String>, third_struct_field: CoreGenericStruct<T, [T]>) {
		self.struct_field = struct_field
		self.second_struct_field = second_struct_field
		self.third_struct_field = third_struct_field
	}
}

public enum CoreEnumUsingGenericStruct: Codable {
	case variantA(CoreGenericStruct<String, Float>)
	case variantB(CoreGenericStruct<String, Int32>)
	case variantC(CoreGenericStruct<String, Bool>)
	case variantD(CoreGenericStructUsingGenericStruct<CodableVoid>)

	enum CodingKeys: String, CodingKey, Codable {
		case variantA = "VariantA",
			variantB = "VariantB",
			variantC = "VariantC",
			variantD = "VariantD"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .variantA:
				if let content = try? container.decode(CoreGenericStruct<String, Float>.self, forKey: .content) {
					self = .variantA(content)
					return
				}
			case .variantB:
				if let content = try? container.decode(CoreGenericStruct<String, Int32>.self, forKey: .content) {
					self = .variantB(content)
					return
				}
			case .variantC:
				if let content = try? container.decode(CoreGenericStruct<String, Bool>.self, forKey: .content) {
					self = .variantC(content)
					return
				}
			case .variantD:
				if let content = try? container.decode(CoreGenericStructUsingGenericStruct<CodableVoid>.self, forKey: .content) {
					self = .variantD(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(CoreEnumUsingGenericStruct.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for CoreEnumUsingGenericStruct"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .variantA(let content):
			try container.encode(CodingKeys.variantA, forKey: .type)
			try container.encode(content, forKey: .content)
		case .variantB(let content):
			try container.encode(CodingKeys.variantB, forKey: .type)
			try container.encode(content, forKey: .content)
		case .variantC(let content):
			try container.encode(CodingKeys.variantC, forKey: .type)
			try container.encode(content, forKey: .content)
		case .variantD(let content):
			try container.encode(CodingKeys.variantD, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}

/// () isn't codable, so we use this instead to represent Rust's unit type
public struct CodableVoid: Codable, Equatable {}
