import Foundation
import AppKit

extension SessionViewModel {

    func addImage(_ data: Data) {
        pendingImages.append(PastedImage(data: data, mediaType: "image/png"))
    }

    func removeImage(_ id: UUID) {
        pendingImages.removeAll { $0.id == id }
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
        status = "\(Status.fetchingPrefix) \(name)…"
        Task { [weak self] in
            do {
                let (data, response) = try await URLSession.shared.data(from: url)
                guard let self else { return }
                let mime = response.mimeType ?? ""
                if mime.hasPrefix("image/"), let image = NSImage(data: data), let png = image.pngData() {
                    self.addImage(png)
                } else {
                    let raw = String(data: data, encoding: .utf8) ?? ""
                    let text = mime.contains("html") ? stripHTML(raw) : raw
                    let doc = ReferenceDoc(name: name, path: link, text: String(text.prefix(Limits.referenceChars)))
                    self.referenceDocs.append(doc)
                    self.referencesDirty = true
                    if persist { self.saveReferences() }
                }
                if self.status.hasPrefix(Status.fetchingPrefix) { self.status = Status.ready }
            } catch {
                self?.status = Status.fetchFailed
            }
        }
    }

    func saveReferences() {
        guard !projectPath.isEmpty else { return }
        ReferenceStore.save(project: projectPath, paths: referenceDocs.map(\.path))
    }

    func loadReferences() {
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
        return Array(matches.prefix(Limits.mentionSuggestions))
    }

    func applyMention(_ path: String) {
        guard let atRange = input.range(of: "@", options: .backwards) else { return }
        input = String(input[..<atRange.lowerBound]) + "@\(path) "
    }

    func rebuildFileIndex() {
        fileIndex = []
        let root = projectPath
        guard !root.isEmpty else { return }
        Task.detached(priority: .utility) { [weak self] in
            let index = SessionViewModel.indexFiles(under: root)
            guard let self else { return }
            await MainActor.run { self.fileIndex = index }
        }
    }

    private nonisolated static func indexFiles(under root: String) -> [String] {
        let base = URL(fileURLWithPath: root)
        let ignored = ["/node_modules/", "/target/", "/.git/", "/build/", "/DerivedData/", "/dist/", "/.next/"]
        guard let walker = FileManager.default.enumerator(
            at: base,
            includingPropertiesForKeys: [.isRegularFileKey],
            options: [.skipsHiddenFiles, .skipsPackageDescendants]
        ) else { return [] }

        var out: [String] = []
        for case let url as URL in walker {
            if out.count >= Limits.fileIndexEntries { break }
            let path = url.path
            if ignored.contains(where: { path.contains($0) }) { continue }
            guard (try? url.resourceValues(forKeys: [.isRegularFileKey]).isRegularFile) == true else { continue }
            out.append(path.replacingOccurrences(of: base.path + "/", with: ""))
        }
        return out
    }
}
