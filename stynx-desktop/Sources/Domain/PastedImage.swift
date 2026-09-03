import Foundation

struct PastedImage: Identifiable {
    let id = UUID()
    let data: Data
    let mediaType: String
}
