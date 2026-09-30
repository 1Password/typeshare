import Foundation

public struct Card: Codable {
	public let name: String

	public init(name: String) {
		self.name = name
	}
}

public typealias Cards = [Card]

public struct Player: Codable {
	public let name: String

	public init(name: String) {
		self.name = name
	}
}

public typealias Players = [Player]

public typealias Pool = Cards
