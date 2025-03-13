import Foundation

public struct OPAddressDetails: Codable {
	public init() {}
}

public enum OPAddress: Codable {
	case fixedAddress(OPAddressDetails)
	case noFixedAddress

	enum CodingKeys: String, CodingKey, Codable {
		case fixedAddress = "FixedAddress",
			noFixedAddress = "NoFixedAddress"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .fixedAddress:
				if let content = try? container.decode(OPAddressDetails.self, forKey: .content) {
					self = .fixedAddress(content)
					return
				}
			case .noFixedAddress:
				self = .noFixedAddress
				return
			}
		}
		throw DecodingError.typeMismatch(OPAddress.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for OPAddress"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .fixedAddress(let content):
			try container.encode(CodingKeys.fixedAddress, forKey: .type)
			try container.encode(content, forKey: .content)
		case .noFixedAddress:
			try container.encode(CodingKeys.noFixedAddress, forKey: .type)
		}
	}
}
