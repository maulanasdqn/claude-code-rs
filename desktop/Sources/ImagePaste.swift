import AppKit

struct PastedImage: Identifiable {
    let id = UUID()
    let data: Data
    let mediaType: String
}

extension NSImage {
    func pngData() -> Data? {
        guard let tiff = tiffRepresentation,
              let rep = NSBitmapImageRep(data: tiff) else { return nil }
        return rep.representation(using: .png, properties: [:])
    }
}

enum ImagePasteboard {
    static func images() -> [Data] {
        let board = NSPasteboard.general
        guard let objects = board.readObjects(forClasses: [NSImage.self], options: nil) as? [NSImage] else {
            return []
        }
        return objects.compactMap { $0.pngData() }
    }

    static func png(from provider: NSItemProvider, completion: @escaping (Data) -> Void) {
        _ = provider.loadObject(ofClass: NSImage.self) { object, _ in
            guard let image = object as? NSImage, let png = image.pngData() else { return }
            completion(png)
        }
    }
}
