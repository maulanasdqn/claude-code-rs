import SwiftUI

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

struct ActionCard: View {
    let tool: ToolItem
    let onOpenFile: (String) -> Void

    private var glyph: (symbol: String, accent: Bool) { toolBadge(for: tool.name) }
    private var isFile: Bool { tool.filePath != nil && ["file_write", "file_edit", "read"].contains(tool.name) }

    var body: some View {
        Button(action: activate) {
            HStack(spacing: 10) {
                Image(systemName: glyph.symbol)
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundStyle(glyph.accent ? Color.accentColor : Color.secondary)
                    .frame(width: 22, height: 22)
                    .background((glyph.accent ? Color.accentColor : Color.secondary).opacity(0.14))
                    .clipShape(RoundedRectangle(cornerRadius: 6, style: .continuous))

                VStack(alignment: .leading, spacing: 2) {
                    Text(displayTitle)
                        .font(.system(.callout, design: .monospaced))
                        .lineLimit(1)
                        .truncationMode(.middle)
                        .foregroundStyle(.primary)
                    Text(tool.name)
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                }

                Spacer(minLength: 6)

                if let stat = tool.stat {
                    Text(stat)
                        .font(.caption.monospacedDigit().weight(.semibold))
                        .foregroundStyle(.green)
                }
                statusBadge
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 9)
            .background(Color.primary.opacity(0.05))
            .overlay(alignment: .leading) {
                Rectangle()
                    .fill(glyph.accent ? Color.accentColor : Color.secondary.opacity(0.5))
                    .frame(width: 3)
            }
            .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }

    private var displayTitle: String {
        tool.title.isEmpty ? tool.name : tool.title
    }

    @ViewBuilder
    private var statusBadge: some View {
        if tool.running {
            ProgressView().controlSize(.small).scaleEffect(0.7)
        } else if tool.isError {
            Image(systemName: "exclamationmark.triangle.fill")
                .font(.caption)
                .foregroundStyle(.orange)
        } else if let badge = tool.badge {
            Text(badge)
                .font(.caption2.weight(.bold))
                .foregroundStyle(.white)
                .frame(width: 16, height: 16)
                .background(badge == "A" ? Color.green : Color.blue)
                .clipShape(RoundedRectangle(cornerRadius: 4, style: .continuous))
        } else {
            Image(systemName: "checkmark")
                .font(.caption2.weight(.bold))
                .foregroundStyle(.green)
        }
    }

    private func activate() {
        if isFile, let path = tool.filePath { onOpenFile(path) }
    }
}
