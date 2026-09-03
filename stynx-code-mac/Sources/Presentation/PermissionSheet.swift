import SwiftUI

struct PermissionSheet: View {
    let prompt: PermissionPrompt
    let respond: (String) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Label("Permission required", systemImage: "lock.shield")
                .font(.headline)

            VStack(alignment: .leading, spacing: 6) {
                Text(prompt.toolName)
                    .font(.system(.subheadline, design: .monospaced))
                    .foregroundStyle(.secondary)
                ScrollView {
                    Text(prompt.description)
                        .font(.system(.callout, design: .monospaced))
                        .textSelection(.enabled)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                .frame(maxHeight: 180)
                .padding(10)
                .background(Color(nsColor: .textBackgroundColor))
                .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
                .overlay(
                    RoundedRectangle(cornerRadius: 8, style: .continuous)
                        .strokeBorder(Color.secondary.opacity(0.2))
                )
            }

            HStack {
                Button("Deny", role: .cancel) { respond("deny") }
                    .keyboardShortcut(.cancelAction)
                Spacer()
                Button("Allow Always") { respond("allow_always") }
                Button("Allow Once") { respond("allow_once") }
                    .keyboardShortcut(.defaultAction)
                    .buttonStyle(.borderedProminent)
            }
        }
        .padding(20)
        .frame(width: 480)
    }
}

struct QuestionSheet: View {
    let question: UserQuestion
    @Binding var draft: String
    let onSubmit: () -> Void
    let onCancel: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Label("Stynx asks", systemImage: "questionmark.bubble")
                .font(.headline)

            ScrollView {
                MarkdownText(raw: question.question)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .frame(maxHeight: 320)

            TextField("Your answer…", text: $draft, axis: .vertical)
                .textFieldStyle(.roundedBorder)
                .lineLimit(1...6)
                .onSubmit(onSubmit)

            HStack {
                Button("Cancel", role: .cancel, action: onCancel)
                    .keyboardShortcut(.cancelAction)
                Spacer()
                Button("Send", action: onSubmit)
                    .keyboardShortcut(.defaultAction)
                    .buttonStyle(.borderedProminent)
                    .disabled(draft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            }
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color.primary.opacity(0.05))
        .overlay(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .strokeBorder(Color.accentColor.opacity(0.35), lineWidth: 1)
        )
        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
    }
}
