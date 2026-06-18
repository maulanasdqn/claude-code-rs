import SwiftUI

struct DiffLineRow: View {
    let line: DiffLine
    let ext: String

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Text(sign)
                .font(.system(.callout, design: .monospaced))
                .foregroundStyle(signColor)
                .frame(width: 12, alignment: .center)
            Text(highlight(line.text, ext: ext))
                .font(.system(.callout, design: .monospaced))
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 1)
        .background(background)
    }

    private var sign: String {
        switch line.kind {
        case .added: return "+"
        case .removed: return "-"
        case .context: return ""
        }
    }

    private var signColor: Color {
        switch line.kind {
        case .added: return .green
        case .removed: return .red
        case .context: return .clear
        }
    }

    private var background: Color {
        switch line.kind {
        case .added: return Color.green.opacity(0.16)
        case .removed: return Color.red.opacity(0.16)
        case .context: return .clear
        }
    }
}

struct CodeLineRow: View {
    let number: Int
    let line: String
    let ext: String

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            Text("\(number)")
                .font(.system(.caption, design: .monospaced))
                .foregroundStyle(.tertiary)
                .frame(width: 34, alignment: .trailing)
            Text(highlight(line, ext: ext))
                .font(.system(.callout, design: .monospaced))
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 1)
    }
}

private let codeKeywords: Set<String> = [
    "fn", "let", "mut", "pub", "struct", "enum", "impl", "trait", "use", "mod", "match",
    "if", "else", "for", "while", "loop", "return", "self", "async", "await", "move",
    "func", "var", "class", "extension", "guard", "import", "static", "private", "public",
    "const", "function", "def", "return", "yield", "true", "false", "nil", "null", "None",
    "where", "in", "as", "is", "try", "throw", "throws", "do", "case", "switch", "default",
]

func highlight(_ line: String, ext: String) -> AttributedString {
    var attributed = AttributedString(line)
    attributed.foregroundColor = .primary

    let trimmed = line.trimmingCharacters(in: .whitespaces)
    if trimmed.hasPrefix("//") || trimmed.hasPrefix("#") || trimmed.hasPrefix("*") {
        attributed.foregroundColor = .secondary
        return attributed
    }

    color(&attributed, in: line, pattern: "\\b[0-9]+(\\.[0-9]+)?\\b", color: Color(red: 0.5, green: 0.8, blue: 0.9))
    for word in codeKeywords {
        color(&attributed, in: line, pattern: "\\b\(NSRegularExpression.escapedPattern(for: word))\\b",
              color: Color(red: 0.85, green: 0.5, blue: 0.85))
    }
    color(&attributed, in: line, pattern: "\"[^\"]*\"", color: Color(red: 0.9, green: 0.6, blue: 0.4))
    return attributed
}

private func color(_ attributed: inout AttributedString, in source: String, pattern: String, color: Color) {
    guard let regex = try? NSRegularExpression(pattern: pattern) else { return }
    let nsRange = NSRange(source.startIndex..., in: source)
    for match in regex.matches(in: source, range: nsRange) {
        guard let swiftRange = Range(match.range, in: source),
              let lower = AttributedString.Index(swiftRange.lowerBound, within: attributed),
              let upper = AttributedString.Index(swiftRange.upperBound, within: attributed)
        else { continue }
        attributed[lower..<upper].foregroundColor = color
    }
}
