import SwiftUI

struct WorkspaceView: View {
    @ObservedObject var model: SessionViewModel
    @State private var showFiles = true

    var body: some View {
        HSplitView {
            ChatView(model: model)
                .frame(minWidth: 400, idealWidth: 600, maxWidth: .infinity)
            if showFiles {
                FilesPanel(model: model)
                    .frame(minWidth: 380, idealWidth: 620, maxWidth: .infinity)
                    .transition(.move(edge: .trailing))
            }
        }
        .navigationTitle("Stynx")
        .navigationSubtitle(model.projectName.isEmpty ? model.modelId : model.projectName)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                statusPill
            }
            ToolbarItemGroup(placement: .primaryAction) {
                sessionHistoryMenu
                Button {
                    model.newSession()
                } label: {
                    Label("New session", systemImage: "square.and.pencil")
                }
                .help("New session")
                .disabled(model.isStreaming)
                Button {
                    withAnimation(.easeInOut(duration: 0.18)) { showFiles.toggle() }
                } label: {
                    Label("Toggle files", systemImage: "sidebar.right")
                }
                .help("Toggle file browser")
            }
        }
        .onChange(of: model.changes.count) {
            if !showFiles {
                withAnimation(.easeInOut(duration: 0.18)) { showFiles = true }
            }
        }
    }

    private var statusPill: some View {
        HStack(spacing: 8) {
            if model.isStreaming {
                ProgressView()
                    .controlSize(.small)
                    .scaleEffect(0.55)
                    .frame(width: 10, height: 10)
            } else {
                Circle()
                    .fill(model.isInitFailed ? Color.red : Color.green)
                    .frame(width: 6, height: 6)
            }
            Text(model.status)
                .font(.caption.weight(.medium))
                .foregroundStyle(.secondary)
                .fixedSize()
        }
        .padding(.horizontal, 22)
        .padding(.vertical, 10)
        .background(.regularMaterial, in: Capsule())
        .overlay(Capsule().strokeBorder(.white.opacity(0.08)))
    }

    private var sessionHistoryMenu: some View {
        Menu {
            if model.sessions.isEmpty {
                Text("No saved sessions")
            } else {
                ForEach(model.sessions, id: \.id) { summary in
                    Button {
                        model.loadSession(summary.id)
                    } label: {
                        Text(summary.title.isEmpty ? "Untitled" : summary.title)
                        Text("\(summary.messageCount) messages")
                    }
                }
            }
        } label: {
            Label("History", systemImage: "clock.arrow.circlepath")
        } primaryAction: {
            model.refreshSessions()
        }
        .menuIndicator(.visible)
        .help("Recent sessions")
        .disabled(model.isStreaming)
    }
}
