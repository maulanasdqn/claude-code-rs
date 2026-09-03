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
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 12) {
                        ForEach(model.changes) { change in
                            DiffCard(
                                change: change,
                                expandedByDefault: change.id == model.changes.last?.id,
                                onQuote: { lines in model.appendDiffQuote(filePath: change.path, lines: lines) }
                            )
                        }
                        Color.clear.frame(height: 1).id("diff-bottom")
                    }
                    .padding(14)
                    .frame(maxWidth: .infinity, alignment: .topLeading)
                }
                .onChange(of: model.changes.count) {
                    withAnimation(.easeOut(duration: 0.2)) {
                        proxy.scrollTo("diff-bottom", anchor: .bottom)
                    }
                }
            }
        }
    }
}

private struct DiffCard: View {
    let change: FileChange
    let expandedByDefault: Bool
    let onQuote: ([DiffLine]) -> Void

    @State private var expanded: Bool?
    @State private var selectedIds: Set<UUID> = []

    private var isExpanded: Bool { expanded ?? expandedByDefault }
    private var ext: String { (change.path as NSString).pathExtension.lowercased() }
    private var allLines: [DiffLine] { change.diff ?? [] }
    private var selectedLines: [DiffLine] {
        allLines.filter { selectedIds.contains($0.id) }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            cardHeader
            if isExpanded {
                Divider()
                diffBody
            }
        }
        .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
        .glassBackground(cornerRadius: 14)
        .overlay(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .strokeBorder(Color.white.opacity(0.08))
        )
    }

    private var cardHeader: some View {
        Button {
            expanded = !isExpanded
            if !isExpanded { selectedIds.removeAll() }
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
                if !selectedLines.isEmpty {
                    quoteButton
                }
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
    }

    private var quoteButton: some View {
        Button {
            onQuote(selectedLines)
            selectedIds.removeAll()
        } label: {
            Label("Quote \(selectedLines.count) line\(selectedLines.count == 1 ? "" : "s")", systemImage: "quote.bubble")
                .font(.caption2.weight(.semibold))
                .padding(.horizontal, 8)
                .padding(.vertical, 3)
                .background(Color.accentColor.opacity(0.2), in: Capsule())
                .foregroundStyle(Color.accentColor)
        }
        .buttonStyle(.plain)
        .transition(.scale(scale: 0.85).combined(with: .opacity))
    }

    private var diffBody: some View {
        VStack(alignment: .leading, spacing: 0) {
            if let diff = change.diff {
                ForEach(diff) { line in
                    SelectableDiffLineRow(
                        line: line,
                        ext: ext,
                        isSelected: selectedIds.contains(line.id)
                    ) { toggle(line.id) }
                }
            } else if let lines = change.lines {
                ForEach(Array(lines.enumerated()), id: \.offset) { index, line in
                    CodeLineRow(number: index + 1, line: line, ext: ext)
                }
            }
        }
        .padding(.vertical, 6)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .textSelection(.enabled)
        .animation(.easeInOut(duration: 0.12), value: selectedIds)
    }

    private var badge: some View {
        Text(change.badge)
            .font(.caption2.weight(.bold))
            .foregroundStyle(.white)
            .frame(width: 16, height: 16)
            .background(change.badge == "A" ? Color.green : Color.blue)
            .clipShape(RoundedRectangle(cornerRadius: 4, style: .continuous))
    }

    private func toggle(_ id: UUID) {
        if selectedIds.contains(id) {
            selectedIds.remove(id)
        } else {
            selectedIds.insert(id)
        }
    }
}


