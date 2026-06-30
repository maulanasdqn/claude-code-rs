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
    let id: UUID
    let text: String
    let images: [PastedImage]

    init(text: String, images: [PastedImage]) {
        self.id = UUID()
        self.text = text
        self.images = images
    }
}

@MainActor
final class SessionViewModel: ObservableObject {
    @Published var feed: [FeedItem] = []
    @Published var input: String = ""
    @Published var isStreaming = false
    @Published var modelId: String = "—"
    @Published var status: String = "Starting…"
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

    var onWorkspaceMessage: ((_ target: String, _ task: String, _ id: UInt64) -> Void)?
    private var externalCompletion: ((String) -> Void)?

    init(path: String) {
        boot(path: path)
    }
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
    private var referencesDirty = false

    private var session: StynxSession?
    private var currentStreamKind: FeedRole?
    private var currentToolId = ""
    private var toolInputBuffers: [String: String] = [:]
    private var promptHistory: [String] = []
    private var historyCursor: Int?
    private var fileIndex: [String] = []

    func currentMention(in text: String) -> String? {
        guard let atRange = text.range(of: "@", options: .backwards) else { return nil }
        let after = text[atRange.upperBound...]
        if after.contains(" ") || after.contains("\n") { return nil }
        if atRange.lowerBound != text.startIndex {
            let before = text[text.index(before: atRange.lowerBound)]
            if !before.isWhitespace { return nil }
        }
        return String(after)
    }

    func mentionSuggestions(_ query: String) -> [String] {
        let matches = query.isEmpty
            ? fileIndex
            : fileIndex.filter { $0.localizedCaseInsensitiveContains(query) }
        return Array(matches.prefix(8))
    }

    func applyMention(_ path: String) {
        guard let atRange = input.range(of: "@", options: .backwards) else { return }
        input = String(input[..<atRange.lowerBound]) + "@\(path) "
    }

    private func buildFileIndex() {
        fileIndex = []
        guard !projectPath.isEmpty else { return }
        let base = URL(fileURLWithPath: projectPath)
        let ignored = ["/node_modules/", "/target/", "/.git/", "/build/", "/DerivedData/", "/dist/", "/.next/"]
        guard let walker = FileManager.default.enumerator(
            at: base,
            includingPropertiesForKeys: [.isRegularFileKey],
            options: [.skipsHiddenFiles, .skipsPackageDescendants]
        ) else { return }

        var out: [String] = []
        for case let url as URL in walker {
            if out.count >= 3000 { break }
            let path = url.path
            if ignored.contains(where: { path.contains($0) }) { continue }
            guard (try? url.resourceValues(forKeys: [.isRegularFileKey]).isRegularFile) == true else { continue }
            out.append(path.replacingOccurrences(of: base.path + "/", with: ""))
        }
        fileIndex = out
    }

    func historyUp() -> Bool {
        guard !promptHistory.isEmpty else { return false }
        if let cursor = historyCursor {
            historyCursor = max(0, cursor - 1)
        } else {
            guard input.isEmpty else { return false }
            historyCursor = promptHistory.count - 1
        }
        input = promptHistory[historyCursor!]
        return true
    }

    func historyDown() -> Bool {
        guard let cursor = historyCursor else { return false }
        let next = cursor + 1
        if next >= promptHistory.count {
            historyCursor = nil
            input = ""
        } else {
            historyCursor = next
            input = promptHistory[next]
        }
        return true
    }

    func switchProvider(_ label: String) {
        guard !isStreaming, label != currentProvider else { return }
        resetTransient()
        boot(path: projectPath.isEmpty ? FileManager.default.currentDirectoryPath : projectPath,
             provider: label)
    }

    func setModel(_ model: String) {
        session?.setModel(model: model)
        modelId = session?.modelId() ?? model
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
            self.status = "Ready"
            buildFileIndex()
            loadReferences()
            refreshSessions()
        } catch {
            self.status = "Init failed"
            feed.append(.assistant("Could not start the engine: \(error)"))
        }
    }

    private func resetTransient() {
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

    func refreshSessions() {
        guard let session else { return }
        sessions = session.listSessions()
    }

    func newSession() {
        guard let session, !isStreaming else { return }
        session.newSession()
        resetTransient()
        status = "Ready"
    }

    func loadSession(_ id: String) {
        guard let session, !isStreaming else { return }
        let turns = session.loadSession(id: id)
        resetTransient()
        feed = turns.map { $0.role == "user" ? FeedItem.user($0.text) : FeedItem.assistant($0.text) }
        status = "Loaded session"
    }

    func deleteSession(_ id: String) {
        guard let session else { return }
        session.deleteSession(id: id)
        refreshSessions()
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
        if status == "Thinking…" { status = "Stopped" }
    }

    func addImage(_ data: Data) {
        pendingImages.append(PastedImage(data: data, mediaType: "image/png"))
    }

    func addReference(_ url: URL) {
        let doc = ReferenceExtractor.load(url)
        referenceDocs.append(doc)
        referencesDirty = true
        saveReferences()
    }

    func removeReference(_ id: UUID) {
        referenceDocs.removeAll { $0.id == id }
        referencesDirty = true
        saveReferences()
    }

    func addReferenceFromURL(_ link: String, persist: Bool = true) {
        guard let url = URL(string: link), url.scheme?.hasPrefix("http") == true else { return }
        let name = url.lastPathComponent.isEmpty ? (url.host ?? link) : url.lastPathComponent
        status = "Fetching \(name)…"
        Task { [weak self] in
            do {
                let (data, response) = try await URLSession.shared.data(from: url)
                let mime = response.mimeType ?? ""
                await MainActor.run {
                    guard let self else { return }
                    if mime.hasPrefix("image/"), let image = NSImage(data: data), let png = image.pngData() {
                        self.addImage(png)
                    } else {
                        let raw = String(data: data, encoding: .utf8) ?? ""
                        let text = mime.contains("html") ? stripHTML(raw) : raw
                        let doc = ReferenceDoc(name: name, path: link, text: String(text.prefix(60_000)))
                        self.referenceDocs.append(doc)
                        self.referencesDirty = true
                        if persist { self.saveReferences() }
                    }
                    if self.status.hasPrefix("Fetching") { self.status = "Ready" }
                }
            } catch {
                await MainActor.run { self?.status = "Fetch failed" }
            }
        }
    }

    private func saveReferences() {
        guard !projectPath.isEmpty else { return }
        ReferenceStore.save(project: projectPath, paths: referenceDocs.map(\.path))
    }

    private func loadReferences() {
        guard !projectPath.isEmpty else { return }
        let paths = ReferenceStore.load(project: projectPath)
        referenceDocs = paths
            .filter { !$0.hasPrefix("http") && FileManager.default.fileExists(atPath: $0) }
            .map { ReferenceExtractor.load(URL(fileURLWithPath: $0)) }
        referencesDirty = !referenceDocs.isEmpty
        for link in paths where link.hasPrefix("http") {
            addReferenceFromURL(link, persist: false)
        }
    }

    func removeImage(_ id: UUID) {
        pendingImages.removeAll { $0.id == id }
    }

    func respondWorkspaceMessage(id: UInt64, reply: String) {
        session?.respondWorkspaceMessage(id: id, reply: reply)
    }

    func runExternalTask(_ task: String, completion: @escaping (String) -> Void) {
        guard let session, !isStreaming else {
            completion("(workspace '\(projectName)' is busy)")
            return
        }
        externalCompletion = completion
        handlingExternal = true
        var incoming = FeedItem(role: .tool, tool: ToolItem(
            toolId: "incoming-\(UUID().uuidString)",
            name: "incoming_workspace",
            title: task,
            running: false
        ))
        incoming.tool?.subtitle = task
        feed.append(incoming)
        currentStreamKind = nil
        isStreaming = true
        status = "Thinking…"
        let bridge = EventBridge { [weak self] event in
            Task { @MainActor in self?.handle(event) }
        }
        session.sendMessage(text: task, listener: bridge)
    }

    func send() {
        guard let session else { return }
        let trimmed = input.trimmingCharacters(in: .whitespacesAndNewlines)
        let images = pendingImages
        guard !trimmed.isEmpty || !images.isEmpty else { return }

        input = ""
        pendingImages = []

        if isStreaming {
            messageQueue.append(QueuedMessage(text: trimmed, images: images))
            return
        }

        dispatchMessage(text: trimmed, images: images)
    }

    func appendDiffQuote(filePath: String, lines: [DiffLine]) {
        let name = (filePath as NSString).lastPathComponent
        let body = lines.map { line in
            switch line.kind {
            case .added:   return "+\(line.text)"
            case .removed: return "-\(line.text)"
            case .context: return " \(line.text)"
            }
        }.joined(separator: "\n")
        let block = "[diff: \(name)]\n```diff\n\(body)\n```\n"
        if input.isEmpty {
            input = block
        } else {
            input += "\n\(block)"
        }
    }

    func dequeueMessage(_ id: UUID) {
        messageQueue.removeAll { $0.id == id }
    }

    private func dispatchMessage(text: String, images: [PastedImage]) {
        guard let session else { return }
        if promptHistory.last != text { promptHistory.append(text) }
        historyCursor = nil
        var userItem = FeedItem.user(text)
        userItem.images = images.map(\.data)
        userItem.referenceCount = (referencesDirty && !referenceDocs.isEmpty) ? referenceDocs.count : 0
        feed.append(userItem)
        currentStreamKind = nil
        isStreaming = true
        status = "Thinking…"

        let bridge = EventBridge { [weak self] event in
            Task { @MainActor in self?.handle(event) }
        }
        let payload = messagePayload(text)
        if images.isEmpty {
            session.sendMessage(text: payload, listener: bridge)
        } else {
            let ffiImages = images.map {
                FfiImage(mediaType: $0.mediaType, data: $0.data.base64EncodedString())
            }
            session.sendMessageWithImages(text: payload, images: ffiImages, listener: bridge)
        }
    }

    private func messagePayload(_ text: String) -> String {
        guard !referenceDocs.isEmpty, referencesDirty else { return text }
        referencesDirty = false
        let blocks = referenceDocs.map { doc -> String in
            let body = doc.hasText ? doc.text : "(no extractable text — \(doc.path))"
            return "## \(doc.name)\n\(body)"
        }
        let refs = blocks.joined(separator: "\n\n")
        let intro = "The user attached the following reference document(s). Treat their content as authoritative context for this and following requests."
        return "\(intro)\n\n<reference_documents>\n\(refs)\n</reference_documents>\n\n\(text)"
    }

    func respondPermission(_ choice: String) {
        guard let prompt = permissionPrompt, let session else { return }
        session.respondPermission(id: prompt.id, choice: choice)
        permissionPrompt = nil
    }

    func submitQuestion() {
        guard let question = userQuestion, let session else { return }
        session.respondAskUser(id: question.id, answer: questionDraft)
        userQuestion = nil
        questionDraft = ""
    }

    func cancelQuestion() {
        guard let question = userQuestion, let session else { return }
        session.respondAskUser(id: question.id, answer: nil)
        userQuestion = nil
        questionDraft = ""
    }

    func answerQuestion(_ text: String) {
        guard let question = userQuestion, let session else { return }
        session.respondAskUser(id: question.id, answer: text)
        userQuestion = nil
        questionDraft = ""
    }

    func skipQuestion() {
        guard let question = userQuestion, let session else { return }
        session.respondAskUser(id: question.id, answer: "(skipped)")
        userQuestion = nil
        questionDraft = ""
    }

    private func handle(_ event: FfiEvent) {
        switch event {
        case .textDelta(let text):
            appendDelta(role: .assistant, delta: text)
        case .thinkingDelta(let text):
            appendDelta(role: .thinking, delta: text)
        case .toolStart(let name, let id):
            currentStreamKind = nil
            currentToolId = id
            toolInputBuffers[id] = ""
            feed.append(.tool(ToolItem(toolId: id, name: name, title: name)))
        case .toolInput(let jsonChunk):
            accumulateToolInput(jsonChunk)
        case .toolOutput(let name, let chunk):
            appendToolOutput(name: name, chunk: chunk)
        case .toolResult(let name, let output, let isError):
            completeTool(name: name, output: output, isError: isError)
            currentStreamKind = nil
        case .usage(let inTokens, let outTokens):
            inputTokens = inTokens
            outputTokens = outTokens
        case .permissionRequest(let id, let toolName, let description):
            permissionPrompt = PermissionPrompt(id: id, toolName: toolName, description: description)
        case .askUserRequest(let id, let question):
            userQuestion = UserQuestion(id: id, question: question, qa: parseQA(question))
        case .workspaceMessageRequest(let id, let target, let task):
            onWorkspaceMessage?(target, task, id)
        case .error(let message):
            currentStreamKind = nil
            feed.append(.assistant("⚠️ \(message)"))
        case .compacted(let originalTurns):
            currentStreamKind = nil
            feed.append(.compact(originalTurns: Int(originalTurns)))
        case .idle:
            isStreaming = false
            currentStreamKind = nil
            if status == "Thinking…" { status = "Ready" }
            fileTreeReloadToken = UUID()
            refreshSessions()
            if let completion = externalCompletion {
                externalCompletion = nil
                handlingExternal = false
                let reply = feed.last(where: { $0.role == .assistant })?.text ?? "(no reply)"
                completion(reply)
            }
            if !messageQueue.isEmpty {
                let next = messageQueue.removeFirst()
                dispatchMessage(text: next.text, images: next.images)
            }
        default:
            break
        }
    }

    private func appendDelta(role: FeedRole, delta: String) {
        if currentStreamKind == role, let last = feed.last, last.role == role {
            feed[feed.count - 1].text += delta
        } else {
            feed.append(FeedItem(role: role, text: delta))
            currentStreamKind = role
        }
    }

    private func accumulateToolInput(_ chunk: String) {
        guard !currentToolId.isEmpty else { return }
        let buffer = (toolInputBuffers[currentToolId] ?? "") + chunk
        toolInputBuffers[currentToolId] = buffer
        guard let index = feed.lastIndex(where: { $0.tool?.toolId == currentToolId }) else { return }
        let name = feed[index].tool?.name ?? ""
        if name == "message_workspace" {
            if let target = toolInputField(buffer, keys: ["target"]) {
                feed[index].tool?.title = target
            }
            if let task = toolInputField(buffer, keys: ["task"]) {
                feed[index].tool?.subtitle = task
            }
            return
        }
        if let title = title(for: name, input: buffer) {
            feed[index].tool?.title = title
        }
        if let path = toolInputField(buffer, keys: ["path", "file_path"]) {
            feed[index].tool?.filePath = absolutePath(path)
        }
    }

    private func appendToolOutput(name: String, chunk: String) {
        guard let index = lastRunningToolIndex(name: name) else { return }
        let combined = (feed[index].tool?.detail ?? "") + chunk
        feed[index].tool?.detail = String(combined.suffix(2000))
    }

    private func completeTool(name: String, output: String, isError: Bool) {
        guard let index = lastRunningToolIndex(name: name) else { return }
        feed[index].tool?.running = false
        feed[index].tool?.isError = isError
        if feed[index].tool?.detail.isEmpty ?? true {
            feed[index].tool?.detail = String(output.prefix(2000))
        }
        applyStat(at: index, name: name, output: output)

        guard ["file_write", "file_edit"].contains(name),
              let path = feed[index].tool?.filePath
        else { return }
        fileTreeReloadToken = UUID()
        guard !isError else { return }

        let buffer = toolInputBuffers[feed[index].tool?.toolId ?? ""] ?? ""
        if name == "file_edit",
           let args = parseToolArgs(buffer),
           let oldString = args["old_string"] as? String,
           let newString = args["new_string"] as? String {
            let diff = computeDiff(
                old: oldString.components(separatedBy: "\n"),
                new: newString.components(separatedBy: "\n")
            )
            changes.append(FileChange(
                path: path,
                kind: name,
                diff: diff,
                adds: diff.filter { $0.kind == .added }.count,
                removes: diff.filter { $0.kind == .removed }.count
            ))
        } else {
            let content = (try? String(contentsOfFile: path, encoding: .utf8)) ?? ""
            let lines = content.components(separatedBy: "\n")
            changes.append(FileChange(path: path, kind: name, lines: lines, adds: lines.count))
        }
    }

    private func applyStat(at index: Int, name: String, output: String) {
        switch name {
        case "file_write":
            if let lines = output.split(separator: " ").compactMap({ Int($0) }).first {
                feed[index].tool?.stat = "+\(lines)"
            }
            feed[index].tool?.badge = "A"
        case "file_edit":
            feed[index].tool?.badge = "M"
        default:
            break
        }
    }

    private func title(for name: String, input: String) -> String? {
        switch name {
        case "bash":
            return toolInputField(input, keys: ["command"]).map { String($0.prefix(80)) }
        case "file_write", "file_edit", "read":
            return toolInputField(input, keys: ["path", "file_path"]).map { ($0 as NSString).lastPathComponent }
        case "glob", "grep":
            return toolInputField(input, keys: ["pattern", "query"])
        case "web_fetch", "web_search":
            return toolInputField(input, keys: ["url", "query"])
        default:
            if name.hasPrefix("delegate_to_") {
                return toolInputField(input, keys: ["task"]).map { String($0.prefix(80)) }
            }
            return nil
        }
    }

    private func absolutePath(_ path: String) -> String {
        if path.hasPrefix("/") { return path }
        return (projectPath as NSString).appendingPathComponent(path)
    }

    private func lastRunningToolIndex(name: String) -> Int? {
        feed.lastIndex { $0.tool?.running == true && $0.tool?.name == name }
            ?? feed.lastIndex { $0.tool?.name == name }
    }
}
