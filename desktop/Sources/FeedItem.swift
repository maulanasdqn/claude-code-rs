import Foundation

enum FeedRole {
    case user
    case assistant
    case thinking
    case tool
}

struct ToolItem {
    var toolId: String
    var name: String
    var title: String = ""
    var detail: String = ""
    var stat: String?
    var badge: String?
    var running = true
    var isError = false
    var filePath: String?
}

struct FeedItem: Identifiable {
    let id = UUID()
    var role: FeedRole
    var text: String = ""
    var tool: ToolItem?
    var images: [Data] = []
    var referenceCount = 0

    static func user(_ text: String) -> FeedItem { FeedItem(role: .user, text: text) }
    static func assistant(_ text: String) -> FeedItem { FeedItem(role: .assistant, text: text) }
    static func thinking(_ text: String) -> FeedItem { FeedItem(role: .thinking, text: text) }
    static func tool(_ tool: ToolItem) -> FeedItem { FeedItem(role: .tool, tool: tool) }
}

func toolBadge(for name: String) -> (symbol: String, accent: Bool) {
    switch name {
    case "file_write": return ("doc.badge.plus", true)
    case "file_edit": return ("pencil", true)
    case "read": return ("doc.text", false)
    case "bash": return ("terminal", false)
    case "glob": return ("magnifyingglass", false)
    case "grep": return ("text.magnifyingglass", false)
    case "web_fetch", "web_search": return ("globe", false)
    case "todo_write", "todo_read": return ("checklist", false)
    default:
        if name.hasPrefix("delegate_to_") { return ("person.2", true) }
        return ("gearshape", false)
    }
}

func toolInputField(_ json: String, keys: [String]) -> String? {
    for key in keys {
        guard let range = json.range(of: "\"\(key)\"") else { continue }
        let after = json[range.upperBound...]
        guard let colon = after.firstIndex(of: ":") else { continue }
        let rest = after[after.index(after: colon)...]
        guard let open = rest.firstIndex(of: "\"") else { continue }
        let start = rest.index(after: open)
        guard let close = rest[start...].firstIndex(of: "\"") else { continue }
        return String(rest[start..<close])
    }
    return nil
}
