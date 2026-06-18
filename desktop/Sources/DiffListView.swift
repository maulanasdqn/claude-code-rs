import SwiftUI

struct DiffListView: View {
    @ObservedObject var model: SessionViewModel

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            content
        }
    }

    private var header: some View {
        HStack(spacing: 6) {
            Image(systemName: "plus.forwardslash.minus")
            Text("Diff")
                .font(.headline)
            Spacer()
            if !model.changes.isEmpty {
                Text("\(model.changes.count)")
                    .font(.caption.monospacedDigit())
                    .foregroundStyle(.secondary)
                    .padding(.horizontal, 7)
                    .padding(.vertical, 2)
                    .background(.quaternary, in: Capsule())
            }
        }
        .padding(14)
    }

    @ViewBuilder
    private var content: some View {
        if model.changes.isEmpty {
            VStack(spacing: 8) {
                Image(systemName: "doc.text.magnifyingglass")
                    .font(.largeTitle)
                    .foregroundStyle(.tertiary)
                Text("No changes yet")
                    .font(.callout)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else {
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 12) {
                    ForEach(model.changes) { change in
                        DiffCard(change: change, expandedByDefault: change.id == model.changes.last?.id)
                    }
                }
                .padding(14)
                .frame(maxWidth: .infinity, alignment: .topLeading)
            }
        }
    }
}

private struct DiffCard: View {
    let change: FileChange
    let expandedByDefault: Bool
    @State private var expanded: Bool?

    private var isExpanded: Bool { expanded ?? expandedByDefault }
    private var ext: String { (change.path as NSString).pathExtension.lowercased() }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Button {
                expanded = !isExpanded
            } label: {
                HStack(spacing: 8) {
                    Image(systemName: isExpanded ? "chevron.down" : "chevron.right")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                    Image(systemName: "doc.text")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    Text(change.name)
                        .font(.system(.callout, design: .monospaced))
                        .lineLimit(1)
                        .truncationMode(.middle)
                    Spacer(minLength: 6)
                    if change.adds > 0 { Text("+\(change.adds)").foregroundStyle(.green) }
                    if change.removes > 0 { Text("-\(change.removes)").foregroundStyle(.red) }
                    badge
                }
                .font(.caption.monospacedDigit().weight(.semibold))
                .padding(.horizontal, 10)
                .padding(.vertical, 8)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            if isExpanded {
                Divider()
                VStack(alignment: .leading, spacing: 0) {
                    if let diff = change.diff {
                        ForEach(diff) { DiffLineRow(line: $0, ext: ext) }
                    } else if let lines = change.lines {
                        ForEach(Array(lines.enumerated()), id: \.offset) { index, line in
                            CodeLineRow(number: index + 1, line: line, ext: ext)
                        }
                    }
                }
                .padding(.vertical, 6)
                .frame(maxWidth: .infinity, alignment: .topLeading)
                .textSelection(.enabled)
            }
        }
        .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
        .glassBackground(cornerRadius: 14)
        .overlay(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .strokeBorder(Color.white.opacity(0.08))
        )
    }

    private var badge: some View {
        Text(change.badge)
            .font(.caption2.weight(.bold))
            .foregroundStyle(.white)
            .frame(width: 16, height: 16)
            .background(change.badge == "A" ? Color.green : Color.blue)
            .clipShape(RoundedRectangle(cornerRadius: 4, style: .continuous))
    }
}
