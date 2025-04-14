import Foundation

public typealias CoreGenericTypeAlias<T> = [T]

public typealias CoreNonGenericAlias = CoreGenericTypeAlias<String?>
