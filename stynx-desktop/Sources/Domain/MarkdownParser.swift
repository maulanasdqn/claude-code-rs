import Foundation

enum MarkdownBlock {
    case code(String)
    case heading(level: Int, text: String)
    case bullet(text: String)
    case numbered(number: String, text: String)
    case paragraph(String)
    case rule
    case table(header: [String], rows: [[String]])
    case blank
}

/// Line-oriented markdown block parser; inline styling is handled by the
/// presentation layer via `AttributedString(markdown:)`.
enum MarkdownParser {
    static func blocks(_ raw: String) -> [MarkdownBlock] {
        let lines = raw.components(separatedBy: "\n")
        var blocks: [MarkdownBlock] = []
        var index = 0

        while index < lines.count {
            let line = lines[index]
            let trimmed = line.trimmingCharacters(in: .whitespaces)

            if trimmed.hasPrefix("```") {
                var code: [String] = []
                index += 1
                while index < lines.count,
                      !lines[index].trimmingCharacters(in: .whitespaces).hasPrefix("```") {
                    code.append(lines[index])
                    index += 1
                }
                index += 1
                blocks.append(.code(code.joined(separator: "\n")))
                continue
            }

            if isHorizontalRule(trimmed) {
                blocks.append(.rule)
                index += 1
                continue
            }

            if isTableRow(trimmed),
               index + 1 < lines.count,
               isTableSeparator(lines[index + 1].trimmingCharacters(in: .whitespaces)) {
                let header = parseTableRow(trimmed)
                var rows: [[String]] = []
                index += 2
                while index < lines.count,
                      isTableRow(lines[index].trimmingCharacters(in: .whitespaces)) {
                    rows.append(parseTableRow(lines[index].trimmingCharacters(in: .whitespaces)))
                    index += 1
                }
                blocks.append(.table(header: header, rows: rows))
                continue
            }

            if trimmed.isEmpty {
                blocks.append(.blank)
            } else if let heading = headingLevel(trimmed) {
                blocks.append(.heading(level: heading.level, text: heading.text))
            } else if let bullet = bulletText(trimmed) {
                blocks.append(.bullet(text: bullet))
            } else if let numbered = numberedText(trimmed) {
                blocks.append(.numbered(number: numbered.number, text: numbered.text))
            } else {
                blocks.append(.paragraph(line))
            }
            index += 1
        }
        return blocks
    }

    private static func isHorizontalRule(_ line: String) -> Bool {
        guard line.count >= 3 else { return false }
        return ["-", "*", "_"].contains { marker in
            line.allSatisfy { String($0) == marker }
        }
    }

    private static func isTableRow(_ line: String) -> Bool {
        line.contains("|") && !line.isEmpty
    }

    private static func isTableSeparator(_ line: String) -> Bool {
        guard line.contains("-") else { return false }
        let cells = parseTableRow(line)
        guard !cells.isEmpty else { return false }
        return cells.allSatisfy { cell in
            !cell.isEmpty && cell.allSatisfy { $0 == "-" || $0 == ":" || $0 == " " }
        }
    }

    private static func parseTableRow(_ line: String) -> [String] {
        var content = line
        if content.hasPrefix("|") { content.removeFirst() }
        if content.hasSuffix("|") { content.removeLast() }
        return content.components(separatedBy: "|").map {
            $0.trimmingCharacters(in: .whitespaces)
        }
    }

    private static func headingLevel(_ line: String) -> (level: Int, text: String)? {
        var level = 0
        var index = line.startIndex
        while index < line.endIndex, line[index] == "#", level < 6 {
            level += 1
            index = line.index(after: index)
        }
        guard level > 0, index < line.endIndex, line[index] == " " else { return nil }
        return (level, String(line[line.index(after: index)...]))
    }

    private static func bulletText(_ line: String) -> String? {
        for marker in ["- ", "* "] where line.hasPrefix(marker) {
            return String(line.dropFirst(marker.count))
        }
        return nil
    }

    private static func numberedText(_ line: String) -> (number: String, text: String)? {
        var digits = ""
        var index = line.startIndex
        while index < line.endIndex, line[index].isNumber {
            digits.append(line[index])
            index = line.index(after: index)
        }
        guard !digits.isEmpty, index < line.endIndex, line[index] == "." else { return nil }
        let afterDot = line.index(after: index)
        guard afterDot < line.endIndex, line[afterDot] == " " else { return nil }
        return (digits, String(line[line.index(after: afterDot)...]))
    }
}
