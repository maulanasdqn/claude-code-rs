import SwiftUI
import UniformTypeIdentifiers

func isPasteableURL(_ text: String) -> Bool {
    let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.contains(" "), !trimmed.contains("\n") else { return false }
    return trimmed.hasPrefix("http://") || trimmed.hasPrefix("https://")
}

struct ChatView: View {
    @ObservedObject var model: SessionViewModel

    var body: some View {
        VStack(spacing: 0) {
            transcript
            composer
        }
        .overlay(alignment: .bottomTrailing) {
            if !model.messageQueue.isEmpty {
                queuePanel
                    .padding(.trailing, 16)
                    .padding(.bottom, 120)
            }
        }
        .background(Color(nsColor: .textBackgroundColor))
    }

    private static let bottomAnchor = "feed-bottom"

    private var transcript: some View {
        ScrollViewReader { proxy in
            ScrollView {
                VStack(alignment: .leading, spacing: 14) {
                    ForEach(model.feed) { item in
                        FeedItemView(item: item) { _ in }
                            .id(item.id)
                    }
                    trailingItems
                    Color.clear.frame(height: 1).id(Self.bottomAnchor)
                }
                .padding(20)
                .padding(.bottom, 24)
                .frame(maxWidth: .infinity, alignment: .leading)
            }
            .onAppear {
                DispatchQueue.main.async {
                    proxy.scrollTo(Self.bottomAnchor, anchor: .bottom)
                }
            }
            .onChange(of: model.feed.count) { scrollToBottom(proxy) }
            .onChange(of: model.feed.last?.text) { scrollToBottom(proxy) }
            .onChange(of: model.isStreaming) { scrollToBottom(proxy) }
            .onChange(of: model.permissionPrompt?.id) { scrollToBottom(proxy) }
            .onChange(of: model.userQuestion?.id) { scrollToBottom(proxy) }
        }
    }

    @ViewBuilder
    private var trailingItems: some View {
        if let prompt = model.permissionPrompt {
            PermissionCard(prompt: prompt) { model.respondPermission($0) }
        }
        if let question = model.userQuestion {
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
        if showTyping {
            TypingDots()
        }
    }

    private func scrollToBottom(_ proxy: ScrollViewProxy) {
        withAnimation(.easeOut(duration: 0.15)) {
            proxy.scrollTo(Self.bottomAnchor, anchor: .bottom)
        }
    }

    private var queuePanel: some View {
        VStack(alignment: .trailing, spacing: 6) {
            HStack(spacing: 6) {
                Text("Queue")
                    .font(.caption2.weight(.semibold))
                    .foregroundStyle(.secondary)
                Text("\(model.messageQueue.count)")
                    .font(.caption2.weight(.bold))
                    .foregroundStyle(.white)
                    .padding(.horizontal, 5)
                    .padding(.vertical, 1)
                    .background(Color.accentColor, in: Capsule())
            }
            VStack(alignment: .leading, spacing: 4) {
                ForEach(model.messageQueue) { queued in
                    HStack(spacing: 8) {
                        Text(queued.text.isEmpty ? "(image)" : queued.text)
                            .font(.caption)
                            .lineLimit(2)
                            .truncationMode(.tail)
                            .foregroundStyle(.primary)
                        Spacer(minLength: 8)
                        Button {
                            model.dequeueMessage(queued.id)
                        } label: {
                            Image(systemName: "xmark")
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                        .buttonStyle(.plain)
                    }
                    .padding(.horizontal, 10)
                    .padding(.vertical, 6)
                    .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 8))
                }
            }
            .frame(maxWidth: 260)
        }
        .padding(10)
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 12))
        .overlay(RoundedRectangle(cornerRadius: 12).strokeBorder(.white.opacity(0.08)))
        .shadow(color: .black.opacity(0.2), radius: 8, y: 4)
        .transition(.scale(scale: 0.9, anchor: .bottomTrailing).combined(with: .opacity))
    }

    private var showTyping: Bool {        guard model.isStreaming else { return false }
        if let last = model.feed.last,
           last.role == .assistant || last.role == .thinking,
           !last.text.isEmpty {
            return false
        }
        return true
    }

    private var composer: some View {
        VStack(alignment: .leading, spacing: 10) {
            if let query = model.currentMention(in: model.input) {
                let suggestions = model.mentionSuggestions(query)
                if !suggestions.isEmpty {
                    VStack(alignment: .leading, spacing: 0) {
                        ForEach(suggestions, id: \.self) { path in
                            Button {
                                model.applyMention(path)
                            } label: {
                                HStack(spacing: 6) {
                                    Image(systemName: "doc").font(.caption).foregroundStyle(.secondary)
                                    Text(path)
                                        .font(.system(.callout, design: .monospaced))
                                        .lineLimit(1)
                                        .truncationMode(.middle)
                                    Spacer(minLength: 0)
                                }
                                .padding(.horizontal, 10)
                                .padding(.vertical, 6)
                                .contentShape(Rectangle())
                            }
                            .buttonStyle(.plain)
                        }
                    }
                    .padding(6)
                    .glassBackground(cornerRadius: 14)
                }
            }

            if !model.pendingImages.isEmpty {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        ForEach(model.pendingImages) { image in
                            thumbnail(image)
                        }
                    }
                    .padding(.horizontal, 2)
                }
                .frame(height: 62)
            }

            TextField("Ask stynx…", text: $model.input, axis: .vertical)
                .textFieldStyle(.plain)
                .font(.body)
                .lineLimit(1...14)
                .frame(minHeight: 24, alignment: .top)
                .onKeyPress(keys: [.return]) { press in
                    if press.modifiers.contains(.shift) {
                        model.input += "\n"
                        return .handled
                    }
                    model.send()
                    return .handled
                }
                .onKeyPress(.upArrow) { model.historyUp() ? .handled : .ignored }
                .onKeyPress(.downArrow) { model.historyDown() ? .handled : .ignored }
                .onKeyPress(keys: ["v"]) { press in
                    let mods = press.modifiers
                    if mods.contains(.control) {
                        let images = ImagePasteboard.images()
                        guard !images.isEmpty else { return .ignored }
                        images.forEach { model.addImage($0) }
                        return .handled
                    }
                    if mods.contains(.command),
                       let pasted = ImagePasteboard.string(),
                       isPasteableURL(pasted) {
                        model.addReferenceFromURL(pasted)
                        return .handled
                    }
                    return .ignored
                }

            HStack(spacing: 8) {
                Button(action: attachFiles) {
                    Image(systemName: "paperclip")
                        .font(.system(size: 16))
                }
                .buttonStyle(.plain)
                .foregroundStyle(.secondary)
                .help("Attach image")

                Spacer()

                if model.isStreaming {
                    Button(action: model.cancel) {
                        Image(systemName: "stop.circle.fill")
                            .font(.system(size: 28))
                            .symbolRenderingMode(.hierarchical)
                    }
                    .buttonStyle(.plain)
                    .foregroundStyle(.red)
                    .help("Stop")
                } else {
                    Button(action: model.send) {
                        Image(systemName: "arrow.up.circle.fill")
                            .font(.system(size: 28))
                            .symbolRenderingMode(.hierarchical)
                    }
                    .buttonStyle(.plain)
                    .foregroundStyle(canSend ? Color.accentColor : Color.secondary)
                    .disabled(!canSend)
                }
            }
        }
        .padding(14)
        .glassBackground(cornerRadius: 22)
        .padding(.horizontal, 16)
        .padding(.top, 8)
        .padding(.bottom, 14)
    }

    private func attachFiles() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        panel.allowsMultipleSelection = true
        panel.allowedContentTypes = [.image]
        panel.prompt = "Attach"
        guard panel.runModal() == .OK else { return }
        for url in panel.urls {
            if let image = NSImage(contentsOf: url), let png = image.pngData() {
                model.addImage(png)
            }
        }
    }

    private func thumbnail(_ image: PastedImage) -> some View {
        ZStack(alignment: .topTrailing) {
            if let nsImage = NSImage(data: image.data) {
                Image(nsImage: nsImage)
                    .resizable()
                    .scaledToFill()
                    .frame(width: 54, height: 54)
                    .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
            }
            Button {
                model.removeImage(image.id)
            } label: {
                Image(systemName: "xmark.circle.fill")
                    .font(.system(size: 14))
                    .symbolRenderingMode(.palette)
                    .foregroundStyle(.white, .black.opacity(0.6))
            }
            .buttonStyle(.plain)
            .offset(x: 5, y: -5)
        }
    }

    private var canSend: Bool {
        guard !model.isStreaming else { return false }
        return !model.input.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            || !model.pendingImages.isEmpty
    }
}

struct FeedItemView: View {
    let item: FeedItem
    let onOpenFile: (String) -> Void

    var body: some View {
        switch item.role {
        case .user:
            HStack {
                Spacer(minLength: 60)
                VStack(alignment: .trailing, spacing: 6) {
                    if !item.images.isEmpty {
                        HStack(spacing: 8) {
                            ForEach(Array(item.images.enumerated()), id: \.offset) { _, data in
                                if let nsImage = NSImage(data: data) {
                                    Image(nsImage: nsImage)
                                        .resizable()
                                        .scaledToFill()
                                        .frame(width: 120, height: 120)
                                        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
                                }
                            }
                        }
                    }
                    if !item.text.isEmpty {
                        Text(item.text)
                            .font(.body)
                            .textSelection(.enabled)
                            .padding(.horizontal, 14)
                            .padding(.vertical, 9)
                            .background(Color.accentColor.opacity(0.9))
                            .foregroundStyle(.white)
                            .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
                    }
                    if item.referenceCount > 0 {
                        Label(
                            "\(item.referenceCount) reference\(item.referenceCount == 1 ? "" : "s")",
                            systemImage: "doc.badge.plus"
                        )
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                    }
                }
            }
            .frame(maxWidth: .infinity, alignment: .trailing)
        case .assistant:
            VStack(alignment: .leading, spacing: 4) {
                roleLabel("Stynx", color: .secondary)
                MarkdownText(raw: item.text)
            }
        case .thinking:
            ThinkingView(text: item.text)
        case .tool:
            if let tool = item.tool {
                if tool.name == "message_workspace" {
                    CrossWorkspaceCard(tool: tool, incoming: false)
                } else if tool.name == "incoming_workspace" {
                    CrossWorkspaceCard(tool: tool, incoming: true)
                } else {
                    ActionCard(tool: tool, onOpenFile: onOpenFile)
                }
            }
        case .compact:
            CompactDivider(originalTurns: Int(item.text) ?? 0)
        }
    }

    private func roleLabel(_ text: String, color: Color) -> some View {
        Text(text.uppercased())
            .font(.caption2.weight(.semibold))
            .tracking(0.6)
            .foregroundStyle(color)
    }
}

struct CompactDivider: View {
    let originalTurns: Int

    var body: some View {
        HStack(spacing: 8) {
            Rectangle()
                .frame(height: 1)
                .foregroundStyle(.quaternary)
            Label("Context compacted · \(originalTurns) turns summarised", systemImage: "arrow.counterclockwise")
                .font(.caption2)
                .foregroundStyle(.tertiary)
                .fixedSize()
            Rectangle()
                .frame(height: 1)
                .foregroundStyle(.quaternary)
        }
        .padding(.vertical, 4)
    }
}
