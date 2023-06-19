import Foundation

public typealias MyTypeA = CodableVoid

public typealias MyTypeB = CodableVoid

/// () isn't codable, so we use this instead to represent Rust's unit type
public struct CodableVoid: Codable {}
