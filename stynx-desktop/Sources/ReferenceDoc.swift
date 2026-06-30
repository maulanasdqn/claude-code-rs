import Foundation
import PDFKit

struct ReferenceDoc: Identifiable {
    let id = UUID()
    let name: String
    let path: String
    let text: String

    var hasText: Bool { !text.isEmpty }
}

enum ReferenceStore {
    private static var fileURL: URL {
        let home = FileManager.default.homeDirectoryForCurrentUser
        let dir = home.appendingPathComponent(".stynx", isDirectory: true)
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir.appendingPathComponent("desktop-references.json")
    }

    private static func readAll() -> [String: [String]] {
        guard let data = try? Data(contentsOf: fileURL),
              let map = try? JSONDecoder().decode([String: [String]].self, from: data) else {
            return [:]
        }
        return map
    }

    static func load(project: String) -> [String] {
        readAll()[project] ?? []
    }

    static func save(project: String, paths: [String]) {
        var map = readAll()
        if paths.isEmpty { map.removeValue(forKey: project) } else { map[project] = paths }
        if let data = try? JSONEncoder().encode(map) {
            try? data.write(to: fileURL)
        }
    }
}

func stripHTML(_ html: String) -> String {
    var s = html
    s = s.replacingOccurrences(of: "<script[^>]*>[\\s\\S]*?</script>", with: " ", options: .regularExpression)
    s = s.replacingOccurrences(of: "<style[^>]*>[\\s\\S]*?</style>", with: " ", options: .regularExpression)
    s = s.replacingOccurrences(of: "<[^>]+>", with: " ", options: .regularExpression)
    s = s.replacingOccurrences(of: "&nbsp;", with: " ")
    s = s.replacingOccurrences(of: "&amp;", with: "&")
    s = s.replacingOccurrences(of: "&lt;", with: "<")
    s = s.replacingOccurrences(of: "&gt;", with: ">")
    s = s.replacingOccurrences(of: "&#39;", with: "'")
    s = s.replacingOccurrences(of: "&quot;", with: "\"")
    s = s.replacingOccurrences(of: "[ \\t]+", with: " ", options: .regularExpression)
    s = s.replacingOccurrences(of: "\\n{3,}", with: "\n\n", options: .regularExpression)
    return s.trimmingCharacters(in: .whitespacesAndNewlines)
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
