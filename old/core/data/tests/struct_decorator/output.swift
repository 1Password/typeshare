import Foundation

public typealias OPBestHockeyTeams5 = String

public struct OPBestHockeyTeams: Codable {
	public let PittsburghPenguins: UInt32
	public let Lies: String

	public init(PittsburghPenguins: UInt32, Lies: String) {
		self.PittsburghPenguins = PittsburghPenguins
		self.Lies = Lies
	}
}

public struct OPBestHockeyTeams1: Codable, Equatable {
	public let PittsburghPenguins: UInt32
	public let Lies: String

	public init(PittsburghPenguins: UInt32, Lies: String) {
		self.PittsburghPenguins = PittsburghPenguins
		self.Lies = Lies
	}
}

public struct OPBestHockeyTeams2: Codable, Comparable, Equatable, Hashable {
	public let PittsburghPenguins: UInt32
	public let Lies: String

	public init(PittsburghPenguins: UInt32, Lies: String) {
		self.PittsburghPenguins = PittsburghPenguins
		self.Lies = Lies
	}
}

public struct OPBestHockeyTeams3: Codable {
	public let PittsburghPenguins: UInt32
	public let Lies: String

	public init(PittsburghPenguins: UInt32, Lies: String) {
		self.PittsburghPenguins = PittsburghPenguins
		self.Lies = Lies
	}
}

public struct OPBestHockeyTeams4: Codable, Equatable, Hashable {
	public let PittsburghPenguins: UInt32
	public let Lies: String

	public init(PittsburghPenguins: UInt32, Lies: String) {
		self.PittsburghPenguins = PittsburghPenguins
		self.Lies = Lies
	}
}
