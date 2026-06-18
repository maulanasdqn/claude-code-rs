import Foundation
import PDFKit

struct ReferenceDoc: Identifiable {
    let id = UUID()
    let name: String
    let path: String
    let text: String

    var hasText: Bool { !text.isEmpty }
}

enum ReferenceExtractor {
    static func load(_ url: URL) -> ReferenceDoc {
        let name = url.lastPathComponent
        let text = extractText(url)
        return ReferenceDoc(name: name, path: url.path, text: text)
    }

    private static func extractText(_ url: URL) -> String {
        let ext = url.pathExtension.lowercased()
        if ext == "pdf" {
            if let doc = PDFDocument(url: url) {
                var out = ""
                for index in 0..<doc.pageCount {
                    if let page = doc.page(at: index), let s = page.string {
                        out += s + "\n"
                    }
                }
                return String(out.prefix(60_000))
            }
            return ""
        }
        if let content = try? String(contentsOf: url, encoding: .utf8) {
            return String(content.prefix(60_000))
        }
        return ""
    }
}
