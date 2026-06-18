import SwiftUI

struct QASheet: View {
    let questions: [QAQuestion]
    let onSend: (String) -> Void
    let onSkip: () -> Void

    @State private var selections: [UUID: [String]] = [:]
    @State private var otherText: [UUID: String] = [:]

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 6) {
                Image(systemName: "bubble.left.and.bubble.right")
                Text("Q&A").font(.headline)
                Spacer()
            }
            .padding(14)
            Divider()

            ScrollView {
                VStack(spacing: 0) {
                    ForEach(Array(questions.enumerated()), id: \.element.id) { index, question in
                        QARow(
                            question: question,
                            selected: selections[question.id] ?? [],
                            other: otherText[question.id] ?? "",
                            onToggle: { toggle(question, $0) },
                            onOther: { otherText[question.id] = $0; selections[question.id] = ["__other__"] }
                        )
                        if index < questions.count - 1 { Divider() }
                    }
                }
            }
            .frame(maxHeight: 380)

            Divider()
            HStack(spacing: 10) {
                Button(action: onSkip) {
                    Text("Skip").frame(maxWidth: .infinity)
                }
                .controlSize(.large)
                Button(action: send) {
                    Text("Send").frame(maxWidth: .infinity)
                }
                .controlSize(.large)
                .buttonStyle(.borderedProminent)
                .disabled(!allAnswered)
            }
            .padding(14)
        }
        .frame(width: 540)
    }

    private var allAnswered: Bool {
        questions.allSatisfy { !(selections[$0.id] ?? []).isEmpty }
    }

    private func toggle(_ question: QAQuestion, _ label: String) {
        var current = selections[question.id] ?? []
        if question.allowsMultiple {
            if let idx = current.firstIndex(of: label) { current.remove(at: idx) } else { current.append(label) }
        } else {
            current = [label]
        }
        selections[question.id] = current
    }

    private func send() {
        let lines = questions.map { question -> String in
            let picked = (selections[question.id] ?? []).map { label -> String in
                label == "__other__" ? (otherText[question.id] ?? "") : label
            }
            return "\(question.header): \(picked.joined(separator: ", "))"
        }
        onSend(lines.joined(separator: "\n"))
    }
}

private struct QARow: View {
    let question: QAQuestion
    let selected: [String]
    let other: String
    let onToggle: (String) -> Void
    let onOther: (String) -> Void

    @State private var showPopover = false

    var body: some View {
        Button {
            showPopover = true
        } label: {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text(question.header)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    Text(displayValue)
                        .font(.body.weight(.semibold))
                        .foregroundStyle(isChosen ? .primary : .secondary)
                }
                Spacer()
                Image(systemName: "chevron.right").font(.caption).foregroundStyle(.secondary)
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 12)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .popover(isPresented: $showPopover, arrowEdge: .leading) {
            QAOptionsPopover(
                question: question,
                selected: selected,
                other: other,
                onPick: { label in
                    onToggle(label)
                    if !question.allowsMultiple { showPopover = false }
                },
                onOther: onOther
            )
        }
    }

    private var isChosen: Bool { !selected.isEmpty }

    private var displayValue: String {
        if selected.contains("__other__") { return other.isEmpty ? "Other…" : other }
        return selected.isEmpty ? "Choose…" : selected.joined(separator: ", ")
    }
}

private struct QAOptionsPopover: View {
    let question: QAQuestion
    let selected: [String]
    let other: String
    let onPick: (String) -> Void
    let onOther: (String) -> Void

    @State private var showOtherField = false
    @State private var draft = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(question.question)
                .font(.headline)
                .fixedSize(horizontal: false, vertical: true)

            ForEach(question.options) { option in
                optionCard(
                    title: option.label,
                    description: option.description,
                    isSelected: selected.contains(option.label)
                ) {
                    showOtherField = false
                    onPick(option.label)
                }
            }

            optionCard(
                title: "Other",
                description: nil,
                isSelected: selected.contains("__other__")
            ) {
                showOtherField = true
                onPick("__other__")
            }

            if showOtherField || selected.contains("__other__") {
                TextField("Type your answer…", text: $draft)
                    .textFieldStyle(.roundedBorder)
                    .onChange(of: draft) { onOther(draft) }
                    .onAppear { draft = other }
            }
        }
        .padding(14)
        .frame(width: 360)
    }

    @ViewBuilder
    private func optionCard(
        title: String,
        description: String?,
        isSelected: Bool,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            VStack(alignment: .leading, spacing: 3) {
                Text(title).font(.body.weight(.semibold))
                if let description {
                    Text(description)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(10)
            .background(isSelected ? Color.accentColor.opacity(0.22) : Color.primary.opacity(0.04))
            .overlay(
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .strokeBorder(isSelected ? Color.accentColor : Color.clear, lineWidth: 1.5)
            )
            .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }
}
