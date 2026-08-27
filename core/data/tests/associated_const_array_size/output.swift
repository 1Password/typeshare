import Foundation

public struct SSiteState: Codable {
	public let mapepistr: [String]

	public init(mapepistr: [String]) {
		self.mapepistr = mapepistr
	}
}
