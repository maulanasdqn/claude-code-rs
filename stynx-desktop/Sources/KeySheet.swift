import SwiftUI

struct KeySheet: View {
    let entry: KeyEntry
    @Binding var key: String
    let onSave: (String) -> Void
    let onCancel: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Label("API key · \(entry.provider)", systemImage: "key.fill")
                .font(.headline)

            VStack(alignment: .leading, spacing: 6) {
                Text("Stored to ~/.stynx/keys.json and exported as")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text(entry.envName)
                    .font(.system(.caption, design: .monospaced))
                    .foregroundStyle(.secondary)
            }

            SecureField("Paste API key…", text: $key)
                .textFieldStyle(.roundedBorder)
                .onSubmit { save() }

            HStack {
                Button("Cancel", role: .cancel, action: onCancel)
                    .keyboardShortcut(.cancelAction)
                Spacer()
                Button("Save", action: save)
                    .keyboardShortcut(.defaultAction)
                    .buttonStyle(.borderedProminent)
                    .disabled(key.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            }
        }
        .padding(20)
        .frame(width: 460)
    }

    private func save() {
        let trimmed = key.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        onSave(trimmed)
    }
}
