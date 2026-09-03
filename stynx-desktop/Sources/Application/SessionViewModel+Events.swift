import Foundation

// MARK: - FFI event stream → feed updates

extension SessionViewModel {
    func handle(_ event: FfiEvent) {
        // Any event other than a retry notice means the stream is alive again.
        if case .retryNotice = event {} else if status.hasPrefix(Status.overloadedPrefix) {
            status = Status.thinking
        }
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
        case .retryNotice(let attempt, let maxAttempts, let delayMs, _):
            status = "\(Status.overloadedPrefix) — retry \(attempt)/\(maxAttempts) in \(delayMs / 1000)s"
        case .error(let message):
            currentStreamKind = nil
            feed.append(.assistant("⚠️ \(message)"))
        case .compacted(let originalTurns):
            currentStreamKind = nil
            feed.append(.compact(originalTurns: Int(originalTurns)))
        case .idle:
            handleIdle()
        default:
            break
        }
    }

    private func handleIdle() {
        isStreaming = false
        currentStreamKind = nil
        if status == Status.thinking { status = Status.ready }
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
        feed[index].tool?.detail = String(combined.suffix(Limits.toolDetailChars))
    }

    private func completeTool(name: String, output: String, isError: Bool) {
        guard let index = lastRunningToolIndex(name: name) else { return }
        feed[index].tool?.running = false
        feed[index].tool?.isError = isError
        if feed[index].tool?.detail.isEmpty ?? true {
            feed[index].tool?.detail = String(output.prefix(Limits.toolDetailChars))
        }
        applyStat(at: index, name: name, output: output)

        guard ["file_write", "file_edit"].contains(name),
              let path = feed[index].tool?.filePath
        else { return }
        fileTreeReloadToken = UUID()
        guard !isError else { return }
        recordFileChange(name: name, path: path, toolId: feed[index].tool?.toolId ?? "")
    }

    private func recordFileChange(name: String, path: String, toolId: String) {
        let buffer = toolInputBuffers[toolId] ?? ""
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
            return toolInputField(input, keys: ["command"]).map { String($0.prefix(Limits.toolTitleChars)) }
        case "file_write", "file_edit", "read":
            return toolInputField(input, keys: ["path", "file_path"]).map { ($0 as NSString).lastPathComponent }
        case "glob", "grep":
            return toolInputField(input, keys: ["pattern", "query"])
        case "web_fetch", "web_search":
            return toolInputField(input, keys: ["url", "query"])
        default:
            if name.hasPrefix("delegate_to_") {
                return toolInputField(input, keys: ["task"]).map { String($0.prefix(Limits.toolTitleChars)) }
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
