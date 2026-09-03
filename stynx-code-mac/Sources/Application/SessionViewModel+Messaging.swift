import Foundation

extension SessionViewModel {
    func send() {
        guard session != nil else { return }
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

    func dequeueMessage(_ id: UUID) {
        messageQueue.removeAll { $0.id == id }
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
        status = Status.thinking
        session.sendMessage(text: task, listener: makeEventBridge())
    }

    func dispatchMessage(text: String, images: [PastedImage]) {
        guard let session else { return }
        if promptHistory.last != text { promptHistory.append(text) }
        historyCursor = nil
        var userItem = FeedItem.user(text)
        userItem.images = images.map(\.data)
        userItem.referenceCount = (referencesDirty && !referenceDocs.isEmpty) ? referenceDocs.count : 0
        feed.append(userItem)
        currentStreamKind = nil
        isStreaming = true
        status = Status.thinking

        let bridge = makeEventBridge()
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

    private func makeEventBridge() -> EventBridge {
        EventBridge { [weak self] event in
            Task { @MainActor in self?.handle(event) }
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

    func historyUp() -> Bool {
        guard !promptHistory.isEmpty else { return false }
        let cursor: Int
        if let current = historyCursor {
            cursor = max(0, current - 1)
        } else {
            guard input.isEmpty else { return false }
            cursor = promptHistory.count - 1
        }
        historyCursor = cursor
        input = promptHistory[cursor]
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

    func respondWorkspaceMessage(id: UInt64, reply: String) {
        session?.respondWorkspaceMessage(id: id, reply: reply)
    }

    func respondPermission(_ choice: String) {
        guard let prompt = permissionPrompt, let session else { return }
        session.respondPermission(id: prompt.id, choice: choice)
        permissionPrompt = nil
    }

    func submitQuestion() {
        respondToQuestion(questionDraft)
    }

    func cancelQuestion() {
        respondToQuestion(nil)
    }

    func answerQuestion(_ text: String) {
        respondToQuestion(text)
    }

    func skipQuestion() {
        respondToQuestion("(skipped)")
    }

    private func respondToQuestion(_ answer: String?) {
        guard let question = userQuestion, let session else { return }
        session.respondAskUser(id: question.id, answer: answer)
        userQuestion = nil
        questionDraft = ""
    }
}
