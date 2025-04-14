import Foundation

public typealias OPBar = String

public struct OPFoo: Codable {
	public let bar: OPBar

	public init(bar: OPBar) {
		self.bar = bar
	}
}
