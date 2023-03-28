import Foundation

public enum CoreGenericEnum<A: Codable, B: Codable>: Codable {
	case variantA(A)
	case variantB(B)

	enum CodingKeys: String, CodingKey, Codable {
		case variantA = "VariantA",
			variantB = "VariantB"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .variantA:
				if let content = try? container.decode(A.self, forKey: .content) {
					self = .variantA(content)
					return
				}
			case .variantB:
				if let content = try? container.decode(B.self, forKey: .content) {
					self = .variantB(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(CoreGenericEnum.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for CoreGenericEnum"))
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
		}
	}
}

public struct CoreStructUsingGenericEnum: Codable {
	public let enum_field: CoreGenericEnum<String, Int16>

	public init(enum_field: CoreGenericEnum<String, Int16>) {
		self.enum_field = enum_field
	}
}

public enum CoreGenericEnumUsingGenericEnum<T: Codable>: Codable {
	case variantC(CoreGenericEnum<T, T>)
	case variantD(CoreGenericEnum<String, [String: T]>)
	case variantE(CoreGenericEnum<String, UInt32>)

	enum CodingKeys: String, CodingKey, Codable {
		case variantC = "VariantC",
			variantD = "VariantD",
			variantE = "VariantE"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .variantC:
				if let content = try? container.decode(CoreGenericEnum<T, T>.self, forKey: .content) {
					self = .variantC(content)
					return
				}
			case .variantD:
				if let content = try? container.decode(CoreGenericEnum<String, [String: T]>.self, forKey: .content) {
					self = .variantD(content)
					return
				}
			case .variantE:
				if let content = try? container.decode(CoreGenericEnum<String, UInt32>.self, forKey: .content) {
					self = .variantE(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(CoreGenericEnumUsingGenericEnum.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for CoreGenericEnumUsingGenericEnum"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .variantC(let content):
			try container.encode(CodingKeys.variantC, forKey: .type)
			try container.encode(content, forKey: .content)
		case .variantD(let content):
			try container.encode(CodingKeys.variantD, forKey: .type)
			try container.encode(content, forKey: .content)
		case .variantE(let content):
			try container.encode(CodingKeys.variantE, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}


/// Generated type representing the anonymous struct variant `VariantF` of the `GenericEnumsUsingStructVariants` Rust enum
public struct CoreGenericEnumsUsingStructVariantsVariantFInner<T: Codable>: Codable {
	public let action: T

	public init(action: T) {
		self.action = action
	}
}

/// Generated type representing the anonymous struct variant `VariantG` of the `GenericEnumsUsingStructVariants` Rust enum
public struct CoreGenericEnumsUsingStructVariantsVariantGInner<T: Codable, U: Codable>: Codable {
	public let action: T
	public let response: U

	public init(action: T, response: U) {
		self.action = action
		self.response = response
	}
}

/// Generated type representing the anonymous struct variant `VariantH` of the `GenericEnumsUsingStructVariants` Rust enum
public struct CoreGenericEnumsUsingStructVariantsVariantHInner: Codable {
	public let non_generic: Int32

	public init(non_generic: Int32) {
		self.non_generic = non_generic
	}
}

/// Generated type representing the anonymous struct variant `VariantI` of the `GenericEnumsUsingStructVariants` Rust enum
public struct CoreGenericEnumsUsingStructVariantsVariantIInner<T: Codable, U: Codable>: Codable {
	public let vec: [T]
	public let action: CoreMyType<T, U>

	public init(vec: [T], action: CoreMyType<T, U>) {
		self.vec = vec
		self.action = action
	}
}
public enum CoreGenericEnumsUsingStructVariants<T: Codable, U: Codable>: Codable {
	case variantF(CoreGenericEnumsUsingStructVariantsVariantFInner<T>)
	case variantG(CoreGenericEnumsUsingStructVariantsVariantGInner<T, U>)
	case variantH(CoreGenericEnumsUsingStructVariantsVariantHInner)
	case variantI(CoreGenericEnumsUsingStructVariantsVariantIInner<T, U>)

	enum CodingKeys: String, CodingKey, Codable {
		case variantF = "VariantF",
			variantG = "VariantG",
			variantH = "VariantH",
			variantI = "VariantI"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .variantF:
				if let content = try? container.decode(CoreGenericEnumsUsingStructVariantsVariantFInner<T>.self, forKey: .content) {
					self = .variantF(content)
					return
				}
			case .variantG:
				if let content = try? container.decode(CoreGenericEnumsUsingStructVariantsVariantGInner<T, U>.self, forKey: .content) {
					self = .variantG(content)
					return
				}
			case .variantH:
				if let content = try? container.decode(CoreGenericEnumsUsingStructVariantsVariantHInner.self, forKey: .content) {
					self = .variantH(content)
					return
				}
			case .variantI:
				if let content = try? container.decode(CoreGenericEnumsUsingStructVariantsVariantIInner<T, U>.self, forKey: .content) {
					self = .variantI(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(CoreGenericEnumsUsingStructVariants.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for CoreGenericEnumsUsingStructVariants"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .variantF(let content):
			try container.encode(CodingKeys.variantF, forKey: .type)
			try container.encode(content, forKey: .content)
		case .variantG(let content):
			try container.encode(CodingKeys.variantG, forKey: .type)
			try container.encode(content, forKey: .content)
		case .variantH(let content):
			try container.encode(CodingKeys.variantH, forKey: .type)
			try container.encode(content, forKey: .content)
		case .variantI(let content):
			try container.encode(CodingKeys.variantI, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}
