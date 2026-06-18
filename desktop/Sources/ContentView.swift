import SwiftUI
import AppKit

struct ContentView: View {
    @StateObject private var model = SessionViewModel()
    @State private var columnVisibility: NavigationSplitViewVisibility = .all
    @State private var showFiles = true

    var body: some View {
        NavigationSplitView(columnVisibility: $columnVisibility) {
            SidebarView(model: model)
                .navigationSplitViewColumnWidth(min: 220, ideal: 260, max: 320)
        } detail: {
            HSplitView {
                ChatView(model: model)
                    .frame(minWidth: 400, idealWidth: 600, maxWidth: .infinity)
                if showFiles {
                    FilesPanel(model: model)
                        .frame(minWidth: 380, idealWidth: 620, maxWidth: .infinity)
                        .transition(.move(edge: .trailing))
                }
            }
        }
        .navigationTitle("Stynx")
        .navigationSubtitle(model.projectName.isEmpty ? model.modelId : model.projectName)
        .toolbar {
            ToolbarItem(placement: .navigation) {
                Button {
                    openProjectPanel()
                } label: {
                    Label("Open project", systemImage: "folder")
                }
                .help("Open project…")
                .disabled(model.isStreaming)
            }
            ToolbarItemGroup(placement: .primaryAction) {
                statusPill
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
        .onAppear { model.start() }
        .onChange(of: model.changes.count) {
            if !showFiles {
                withAnimation(.easeInOut(duration: 0.18)) { showFiles = true }
            }
        }
        .sheet(item: $model.userQuestion) { question in
            if let qa = question.qa, !qa.isEmpty {
                QASheet(
                    questions: qa,
                    onSend: { model.answerQuestion($0) },
                    onSkip: model.skipQuestion
                )
            } else {
                QuestionSheet(
                    question: question,
                    draft: $model.questionDraft,
                    onSubmit: model.submitQuestion,
                    onCancel: model.cancelQuestion
                )
            }
        }
    }

    private var statusPill: some View {
        HStack(spacing: 6) {
            if model.isStreaming {
                ProgressView()
                    .controlSize(.small)
                    .scaleEffect(0.55)
                    .frame(width: 10, height: 10)
            } else {
                Circle()
                    .fill(statusColor)
                    .frame(width: 6, height: 6)
            }
            Text(model.status)
                .font(.caption.weight(.medium))
                .foregroundStyle(.secondary)
                .fixedSize()
        }
        .padding(.horizontal, 11)
        .padding(.vertical, 5)
        .background(.regularMaterial, in: Capsule())
        .overlay(
            Capsule().strokeBorder(.white.opacity(0.08))
        )
        .padding(.trailing, 10)
    }

    private var statusColor: Color {
        if model.status.hasPrefix("Init failed") { return .red }
        return .green
    }

    private func openProjectPanel() {
        let panel = NSOpenPanel()
        panel.canChooseDirectories = true
        panel.canChooseFiles = false
        panel.allowsMultipleSelection = false
        panel.prompt = "Open"
        panel.message = "Choose a project folder"
        if panel.runModal() == .OK, let url = panel.url {
            model.openProject(path: url.path)
        }
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
