import SwiftUI

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

struct MarkdownText: View {
    let raw: String

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            ForEach(Array(parseBlocks(raw).enumerated()), id: \.offset) { _, block in
                view(for: block)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .textSelection(.enabled)
    }

    @ViewBuilder
    private func view(for block: MarkdownBlock) -> some View {
        switch block {
        case .code(let code):
            Text(code)
                .font(.system(.callout, design: .monospaced))
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(10)
                .background(Color(nsColor: .textBackgroundColor))
                .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
                .overlay(
                    RoundedRectangle(cornerRadius: 8, style: .continuous)
                        .strokeBorder(Color.secondary.opacity(0.18))
                )
        case .heading(let level, let text):
            inline(text)
                .font(.system(level <= 2 ? .title3 : .headline).weight(.semibold))
        case .bullet(let text):
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text("•").foregroundStyle(.secondary)
                inline(text).frame(maxWidth: .infinity, alignment: .leading)
            }
        case .numbered(let number, let text):
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text("\(number).").foregroundStyle(.secondary).monospacedDigit()
                inline(text).frame(maxWidth: .infinity, alignment: .leading)
            }
        case .paragraph(let text):
            inline(text).frame(maxWidth: .infinity, alignment: .leading)
        case .rule:
            Divider().padding(.vertical, 4)
        case .table(let header, let rows):
            TableView(header: header, rows: rows, inline: inline)
        case .blank:
            Color.clear.frame(height: 2)
        }
    }

    private func inline(_ text: String) -> Text {
        let options = AttributedString.MarkdownParsingOptions(
            interpretedSyntax: .inlineOnlyPreservingWhitespace
        )
        if let attributed = try? AttributedString(markdown: text, options: options) {
            return Text(attributed)
        }
        return Text(text)
    }
}

private struct TableView: View {
    let header: [String]
    let rows: [[String]]
    let inline: (String) -> Text

    var body: some View {
        Grid(alignment: .topLeading, horizontalSpacing: 16, verticalSpacing: 6) {
            GridRow {
                ForEach(header.indices, id: \.self) { column in
                    inline(header[column]).font(.callout.weight(.semibold))
                }
            }
            Divider().gridCellColumns(max(header.count, 1))
            ForEach(rows.indices, id: \.self) { row in
                GridRow {
                    ForEach(rows[row].indices, id: \.self) { column in
                        inline(rows[row][column]).font(.callout)
                    }
                }
            }
        }
        .padding(10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .textBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .strokeBorder(Color.secondary.opacity(0.18))
        )
    }
}

private func parseBlocks(_ raw: String) -> [MarkdownBlock] {
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

private func isHorizontalRule(_ line: String) -> Bool {
    guard line.count >= 3 else { return false }
    return ["-", "*", "_"].contains { marker in
        line.allSatisfy { String($0) == marker }
    }
}

private func isTableRow(_ line: String) -> Bool {
    line.contains("|") && !line.isEmpty
}

private func isTableSeparator(_ line: String) -> Bool {
    guard line.contains("-") else { return false }
    let cells = parseTableRow(line)
    guard !cells.isEmpty else { return false }
    return cells.allSatisfy { cell in
        !cell.isEmpty && cell.allSatisfy { $0 == "-" || $0 == ":" || $0 == " " }
    }
}

private func parseTableRow(_ line: String) -> [String] {
    var content = line
    if content.hasPrefix("|") { content.removeFirst() }
    if content.hasSuffix("|") { content.removeLast() }
    return content.components(separatedBy: "|").map {
        $0.trimmingCharacters(in: .whitespaces)
    }
}

private func headingLevel(_ line: String) -> (level: Int, text: String)? {
    var level = 0
    var index = line.startIndex
    while index < line.endIndex, line[index] == "#", level < 6 {
        level += 1
        index = line.index(after: index)
    }
    guard level > 0, index < line.endIndex, line[index] == " " else { return nil }
    return (level, String(line[line.index(after: index)...]))
}

private func bulletText(_ line: String) -> String? {
    for marker in ["- ", "* "] where line.hasPrefix(marker) {
        return String(line.dropFirst(marker.count))
    }
    return nil
}

private func numberedText(_ line: String) -> (number: String, text: String)? {
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
