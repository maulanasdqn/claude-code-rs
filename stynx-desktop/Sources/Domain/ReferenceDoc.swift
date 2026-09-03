import Foundation

struct ReferenceDoc: Identifiable {
    let id = UUID()
    let name: String
    let path: String
    let text: String

    var hasText: Bool { !text.isEmpty }
}
