import Foundation
import AppKit

struct PermissionPrompt: Identifiable {
    let id: UInt64
    let toolName: String
    let description: String
}

struct UserQuestion: Identifiable {
    let id: UInt64
    let question: String
    var qa: [QAQuestion]?
}

struct QueuedMessage: Identifiable {
    let id = UUID()
    let text: String
    let images: [PastedImage]
}

/// Drives one workspace session: owns the FFI session, the feed shown in the
/// chat transcript, and every user-adjustable setting. Split across focused
/// files: messaging (`+Messaging`), FFI event handling (`+Events`), and
/// references/mentions/images (`+References`).
@MainActor
final class SessionViewModel: ObservableObject {
    enum Status {
        static let starting = "Starting…"
        static let ready = "Ready"
        static let thinking = "Thinking…"
        static let stopped = "Stopped"
        static let loadedSession = "Loaded session"
        static let initFailed = "Init failed"
        static let fetchFailed = "Fetch failed"
        static let overloadedPrefix = "Overloaded"
        static let fetchingPrefix = "Fetching"
    }

    enum Limits {
        static let fileIndexEntries = 3000
        static let mentionSuggestions = 8
        static let toolDetailChars = 2000
        static let toolTitleChars = 80
        static let referenceChars = 60_000
    }

    // MARK: - Published state

    @Published var feed: [FeedItem] = []
    @Published var input: String = ""
    @Published var isStreaming = false
    @Published var modelId: String = "—"
    @Published var status: String = Status.starting
    @Published var inputTokens: UInt64 = 0
    @Published var outputTokens: UInt64 = 0
    @Published var mode: PermissionModeOption = .normal
    @Published var permissionPrompt: PermissionPrompt?
    @Published var userQuestion: UserQuestion?
    @Published var questionDraft: String = ""
    @Published var sessions: [FfiSessionSummary] = []
    @Published var interns: [FfiInternInfo] = []
    @Published var projectName: String = ""
    @Published var projectPath: String = ""
    @Published var handlingExternal = false
    @Published var messageQueue: [QueuedMessage] = []
    @Published var thinkingEnabled = true
    @Published var claudeAvailable = false
    @Published var claudeModels: [String] = []
    @Published var deepseekModels: [String] = []
    @Published var currentProvider = ""
    @Published var mainProviders: [String] = []
    @Published var fileTreeReloadToken = UUID()
    @Published var changes: [FileChange] = []
    @Published var pendingImages: [PastedImage] = []
    @Published var referenceDocs: [ReferenceDoc] = []

    var onWorkspaceMessage: ((_ target: String, _ task: String, _ id: UInt64) -> Void)?

    var isInitFailed: Bool { status == Status.initFailed }

    // MARK: - Internal state (shared with same-module extensions)

    var session: StynxSession?
    var currentStreamKind: FeedRole?
    var currentToolId = ""
    var toolInputBuffers: [String: String] = [:]
    var promptHistory: [String] = []
    var historyCursor: Int?
    var fileIndex: [String] = []
    var referencesDirty = false
    var externalCompletion: ((String) -> Void)?

    // MARK: - Lifecycle

    init(path: String) {
        boot(path: path)
    }

    func switchProvider(_ label: String) {
        guard !isStreaming, label != currentProvider else { return }
        resetTransient()
        boot(path: projectPath.isEmpty ? FileManager.default.currentDirectoryPath : projectPath,
             provider: label)
    }

    private func boot(path: String, provider: String? = nil) {
        do {
            let session: StynxSession
            if let provider {
                session = try StynxSession.newWithProvider(workspacePath: path, provider: provider)
            } else {
                session = try StynxSession(workspacePath: path)
            }
            self.session = session
            self.modelId = session.modelId()
            self.interns = session.listInterns()
            self.claudeAvailable = session.claudeAvailable()
            self.claudeModels = session.claudeModels()
            self.deepseekModels = session.deepseekModels()
            self.currentProvider = session.currentProvider()
            self.mainProviders = session.mainProviders()
            self.projectPath = path
            self.projectName = (path as NSString).lastPathComponent
            self.thinkingEnabled = session.thinkingEnabled()
            self.status = Status.ready
            rebuildFileIndex()
            loadReferences()
            refreshSessions()
        } catch {
            self.status = Status.initFailed
            feed.append(.assistant("Could not start the engine: \(error)"))
        }
    }

    func resetTransient() {
        feed.removeAll()
        currentStreamKind = nil
        currentToolId = ""
        toolInputBuffers.removeAll()
        inputTokens = 0
        outputTokens = 0
        changes.removeAll()
        referencesDirty = !referenceDocs.isEmpty
        fileTreeReloadToken = UUID()
    }

    // MARK: - Sessions

    func refreshSessions() {
        guard let session else { return }
        sessions = session.listSessions()
    }

    func newSession() {
        guard let session, !isStreaming else { return }
        session.newSession()
        resetTransient()
        status = Status.ready
    }

    func loadSession(_ id: String) {
        guard let session, !isStreaming else { return }
        let turns = session.loadSession(id: id)
        resetTransient()
        feed = turns.map { $0.role == "user" ? FeedItem.user($0.text) : FeedItem.assistant($0.text) }
        status = Status.loadedSession
    }

    func deleteSession(_ id: String) {
        guard let session else { return }
        session.deleteSession(id: id)
        refreshSessions()
    }

    // MARK: - Settings

    func setModel(_ model: String) {
        session?.setModel(model: model)
        modelId = session?.modelId() ?? model
    }

    func setMode(_ option: PermissionModeOption) {
        mode = option
        session?.setMode(mode: option.rawValue)
    }

    func setThinking(_ enabled: Bool) {
        thinkingEnabled = enabled
        session?.setThinking(enabled: enabled)
    }

    func setProviderKey(envName: String, value: String) {
        guard let session else { return }
        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        _ = session.setProviderKey(envName: envName, value: trimmed)
        interns = session.listInterns()
    }

    func cancel() {
        session?.cancel()
        isStreaming = false
        currentStreamKind = nil
        if status == Status.thinking { status = Status.stopped }
    }
}
