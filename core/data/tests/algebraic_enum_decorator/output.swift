import Foundation

public enum OPBestHockeyTeams: Codable {
	case pittsburghPenguins
	case lies(String)

	enum CodingKeys: String, CodingKey, Codable {
		case pittsburghPenguins = "PittsburghPenguins",
			lies = "Lies"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .pittsburghPenguins:
				self = .pittsburghPenguins
				return
			case .lies:
				if let content = try? container.decode(String.self, forKey: .content) {
					self = .lies(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(OPBestHockeyTeams.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for OPBestHockeyTeams"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .pittsburghPenguins:
			try container.encode(CodingKeys.pittsburghPenguins, forKey: .type)
		case .lies(let content):
			try container.encode(CodingKeys.lies, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}

public enum OPBestHockeyTeams1: Codable, Equatable {
	case pittsburghPenguins
	case lies(String)

	enum CodingKeys: String, CodingKey, Codable {
		case pittsburghPenguins = "PittsburghPenguins",
			lies = "Lies"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .pittsburghPenguins:
				self = .pittsburghPenguins
				return
			case .lies:
				if let content = try? container.decode(String.self, forKey: .content) {
					self = .lies(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(OPBestHockeyTeams1.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for OPBestHockeyTeams1"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .pittsburghPenguins:
			try container.encode(CodingKeys.pittsburghPenguins, forKey: .type)
		case .lies(let content):
			try container.encode(CodingKeys.lies, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}

public enum OPBestHockeyTeams2: Codable, Comparable, Equatable, Hashable {
	case pittsburghPenguins
	case lies(String)

	enum CodingKeys: String, CodingKey, Codable {
		case pittsburghPenguins = "PittsburghPenguins",
			lies = "Lies"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .pittsburghPenguins:
				self = .pittsburghPenguins
				return
			case .lies:
				if let content = try? container.decode(String.self, forKey: .content) {
					self = .lies(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(OPBestHockeyTeams2.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for OPBestHockeyTeams2"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .pittsburghPenguins:
			try container.encode(CodingKeys.pittsburghPenguins, forKey: .type)
		case .lies(let content):
			try container.encode(CodingKeys.lies, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}

public enum OPBestHockeyTeams3: Codable {
	case pittsburghPenguins
	case lies(String)

	enum CodingKeys: String, CodingKey, Codable {
		case pittsburghPenguins = "PittsburghPenguins",
			lies = "Lies"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .pittsburghPenguins:
				self = .pittsburghPenguins
				return
			case .lies:
				if let content = try? container.decode(String.self, forKey: .content) {
					self = .lies(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(OPBestHockeyTeams3.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for OPBestHockeyTeams3"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .pittsburghPenguins:
			try container.encode(CodingKeys.pittsburghPenguins, forKey: .type)
		case .lies(let content):
			try container.encode(CodingKeys.lies, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}

public enum OPBestHockeyTeams4: Codable, Equatable, Hashable {
	case pittsburghPenguins
	case lies(String)

	enum CodingKeys: String, CodingKey, Codable {
		case pittsburghPenguins = "PittsburghPenguins",
			lies = "Lies"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .pittsburghPenguins:
				self = .pittsburghPenguins
				return
			case .lies:
				if let content = try? container.decode(String.self, forKey: .content) {
					self = .lies(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(OPBestHockeyTeams4.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for OPBestHockeyTeams4"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .pittsburghPenguins:
			try container.encode(CodingKeys.pittsburghPenguins, forKey: .type)
		case .lies(let content):
			try container.encode(CodingKeys.lies, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}


/// Generated type representing the anonymous struct variant `MyVariant` of the `MyEnum` Rust enum
public struct OPMyEnumMyVariantInner: Codable, Equatable {
	public let a: String
	public let b: Double

	public init(a: String, b: Double) {
		self.a = a
		self.b = b
	}
}
public enum OPMyEnum: Codable, Equatable {
	case myVariant(OPMyEnumMyVariantInner)

	enum CodingKeys: String, CodingKey, Codable {
		case myVariant = "MyVariant"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .myVariant:
				if let content = try? container.decode(OPMyEnumMyVariantInner.self, forKey: .content) {
					self = .myVariant(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(OPMyEnum.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for OPMyEnum"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .myVariant(let content):
			try container.encode(CodingKeys.myVariant, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}
