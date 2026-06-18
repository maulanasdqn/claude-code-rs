import SwiftUI
import AppKit

struct KeyEntry: Identifiable {
    let id: String
    let provider: String
    let envName: String
}

struct SidebarView: View {
    @ObservedObject var model: SessionViewModel
    @State private var keyEntry: KeyEntry?
    @State private var keyDraft = ""

    var body: some View {
        List {
            Section("Project") {
                LabeledContent("Folder", value: model.projectName.isEmpty ? "—" : model.projectName)
                    .help(model.projectPath)
            }

            Section {
                Button {
                    model.newSession()
                } label: {
                    Label("New conversation", systemImage: "plus.bubble")
                }
                .buttonStyle(.plain)
                .disabled(model.isStreaming)

                if model.sessions.isEmpty {
                    Text("No saved sessions")
                        .font(.callout)
                        .foregroundStyle(.tertiary)
                } else {
                    ForEach(model.sessions, id: \.id) { summary in
                        SessionRow(summary: summary) {
                            model.loadSession(summary.id)
                        } onDelete: {
                            model.deleteSession(summary.id)
                        }
                    }
                }
            } header: {
                Text("Sessions")
            }

            Section("Model") {
                Picker("Main agent", selection: providerBinding) {
                    ForEach(model.mainProviders, id: \.self) { provider in
                        Text(provider).tag(provider)
                    }
                }
                .pickerStyle(.menu)
                .disabled(model.isStreaming)

                if model.currentProvider == "claude" {
                    Picker("Claude model", selection: modelBinding) {
                        ForEach(model.claudeModels, id: \.self) { id in
                            Text(id).tag(id)
                        }
                    }
                    .pickerStyle(.menu)
                } else {
                    ModelEditor(model: model)
                }

                LabeledContent("Status", value: model.status)
                LabeledContent("Tokens", value: "in \(model.inputTokens) · out \(model.outputTokens)")
            }

            Section("Permission") {
                Picker("Mode", selection: modeBinding) {
                    ForEach(PermissionModeOption.allCases) { option in
                        Label(option.label, systemImage: option.symbol).tag(option)
                    }
                }
                .pickerStyle(.menu)
                Toggle("Thinking", isOn: thinkingBinding)
            }

            Section {
                Button(action: addReference) {
                    Label("Add reference", systemImage: "doc.badge.plus")
                }
                .buttonStyle(.plain)

                if model.referenceDocs.isEmpty {
                    Text("No reference documents")
                        .font(.callout)
                        .foregroundStyle(.tertiary)
                } else {
                    ForEach(model.referenceDocs) { doc in
                        ReferenceRow(doc: doc) { model.removeReference(doc.id) }
                    }
                }
            } header: {
                Text("References")
            }

            Section("Interns") {
                if model.interns.isEmpty {
                    Text("No interns configured")
                        .font(.callout)
                        .foregroundStyle(.tertiary)
                } else {
                    ForEach(model.interns, id: \.name) { intern in
                        InternRow(intern: intern) {
                            keyDraft = ""
                            keyEntry = KeyEntry(id: intern.name, provider: intern.provider, envName: intern.keyEnv)
                        }
                    }
                }
            }

        }
        .listStyle(.sidebar)
        .sheet(item: $keyEntry) { entry in
            KeySheet(entry: entry, key: $keyDraft) { value in
                model.setProviderKey(envName: entry.envName, value: value)
                keyEntry = nil
            } onCancel: {
                keyEntry = nil
            }
        }
    }

    private var modeBinding: Binding<PermissionModeOption> {
        Binding(get: { model.mode }, set: { model.setMode($0) })
    }

    private var thinkingBinding: Binding<Bool> {
        Binding(get: { model.thinkingEnabled }, set: { model.setThinking($0) })
    }

    private var providerBinding: Binding<String> {
        Binding(get: { model.currentProvider }, set: { model.switchProvider($0) })
    }

    private var modelBinding: Binding<String> {
        Binding(get: { model.modelId }, set: { model.setModel($0) })
    }

    private func addReference() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        panel.allowsMultipleSelection = true
        panel.prompt = "Add"
        panel.message = "Choose reference documents"
        guard panel.runModal() == .OK else { return }
        for url in panel.urls { model.addReference(url) }
    }
}

private struct ModelEditor: View {
    @ObservedObject var model: SessionViewModel
    @State private var draft = ""

    var body: some View {
        HStack(spacing: 6) {
            Text("Model")
            Spacer()
            TextField("model id", text: $draft)
                .multilineTextAlignment(.trailing)
                .textFieldStyle(.plain)
                .font(.system(.body, design: .monospaced))
                .onSubmit { model.setModel(draft.trimmingCharacters(in: .whitespaces)) }
        }
        .onAppear { draft = model.modelId }
        .onChange(of: model.modelId) { draft = model.modelId }
        .onChange(of: model.currentProvider) { draft = model.modelId }
    }
}

private struct ReferenceRow: View {
    let doc: ReferenceDoc
    let onRemove: () -> Void

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: icon)
                .font(.caption)
                .foregroundStyle(doc.hasText ? Color.accentColor : Color.secondary)
            VStack(alignment: .leading, spacing: 1) {
                Text(doc.name)
                    .font(.callout)
                    .lineLimit(1)
                    .truncationMode(.middle)
                Text(doc.hasText ? "reference" : "no text extracted")
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }
            Spacer(minLength: 0)
            Button(action: onRemove) {
                Image(systemName: "xmark.circle.fill")
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
        }
        .padding(.vertical, 2)
        .help(doc.path)
    }

    private var icon: String {
        switch (doc.name as NSString).pathExtension.lowercased() {
        case "pdf": return "doc.richtext"
        case "doc", "docx": return "doc.text"
        case "md", "txt": return "doc.plaintext"
        default: return "doc"
        }
    }
}

private struct SessionRow: View {
    let summary: FfiSessionSummary
    let onSelect: () -> Void
    let onDelete: () -> Void

    var body: some View {
        Button(action: onSelect) {
            HStack(spacing: 8) {
                Image(systemName: "bubble.left.and.text.bubble.right")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                VStack(alignment: .leading, spacing: 1) {
                    Text(summary.title.isEmpty ? "Untitled" : summary.title)
                        .font(.callout)
                        .lineLimit(1)
                        .truncationMode(.tail)
                    Text("\(summary.messageCount) messages")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                }
                Spacer(minLength: 0)
            }
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .contextMenu {
            Button("Open", action: onSelect)
            Button("Delete", role: .destructive, action: onDelete)
        }
    }
}

private struct InternRow: View {
    let intern: FfiInternInfo
    let onAddKey: () -> Void

    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            Image(systemName: intern.available ? "person.fill.checkmark" : "person.slash")
                .foregroundStyle(intern.available ? Color.green : Color.secondary)
                .font(.caption)
            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text(intern.name)
                        .font(.system(.callout, design: .monospaced))
                    Spacer(minLength: 0)
                    if intern.available {
                        Text("ready")
                            .font(.caption2)
                            .foregroundStyle(.green)
                    } else {
                        Button(action: onAddKey) {
                            Label("Add key", systemImage: "key")
                                .labelStyle(.iconOnly)
                                .font(.caption)
                        }
                        .buttonStyle(.plain)
                        .foregroundStyle(.blue)
                        .help("Set \(intern.keyEnv)")
                    }
                }
                Text(intern.description)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
                Text("\(intern.provider) · \(intern.model)")
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }
        }
        .padding(.vertical, 2)
    }
}
