import SwiftUI

struct PermissionCard: View {
    let prompt: PermissionPrompt
    let respond: (String) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Label("Permission required", systemImage: "lock.shield")
                .font(.subheadline.weight(.semibold))

            Text(prompt.toolName)
                .font(.system(.callout, design: .monospaced))
                .foregroundStyle(.secondary)

            if !prompt.description.isEmpty {
                ScrollView {
                    MarkdownText(raw: prompt.description)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                .frame(maxHeight: 160)
                .padding(10)
                .background(Color(nsColor: .textBackgroundColor))
                .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
            }

            HStack {
                Button("Deny") { respond("deny") }
                Spacer()
                Button("Allow Always") { respond("allow_always") }
                Button("Allow Once") { respond("allow_once") }
                    .buttonStyle(.borderedProminent)
                    .keyboardShortcut(.defaultAction)
            }
        }
        .padding(14)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color.primary.opacity(0.05))
        .overlay(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .strokeBorder(Color.orange.opacity(0.4), lineWidth: 1)
        )
        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
    }
}
