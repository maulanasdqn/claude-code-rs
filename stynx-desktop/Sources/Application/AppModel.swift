import Foundation
import AppKit

@MainActor
final class AppModel: ObservableObject {
    @Published var current: SessionViewModel
    @Published var recentProjects: [String] = []

    private var runtimes: [String: SessionViewModel] = [:]
    private let lastProjectKey = "stynx.lastProjectPath"
    private let recentProjectsKey = "stynx.recentProjects"

    init() {
        let defaults = UserDefaults.standard
        let recents = defaults.stringArray(forKey: recentProjectsKey) ?? []
        let saved = defaults.string(forKey: lastProjectKey)
        let cwd = FileManager.default.currentDirectoryPath
        let path = saved.flatMap { FileManager.default.fileExists(atPath: $0) ? $0 : nil } ?? cwd

        FileManager.default.changeCurrentDirectoryPath(path)
        let runtime = SessionViewModel(path: path)
        self.current = runtime
        self.recentProjects = recents
        self.runtimes[path] = runtime
        wire(runtime)
        remember(path)
        defaults.set(path, forKey: lastProjectKey)
    }

    func switchTo(_ path: String) {
        guard FileManager.default.fileExists(atPath: path) else { return }
        FileManager.default.changeCurrentDirectoryPath(path)
        UserDefaults.standard.set(path, forKey: lastProjectKey)
        remember(path)
        current = runtime(for: path)
    }

    func isOpen(_ path: String) -> Bool { runtimes[path] != nil }

    func removeFromRecents(_ path: String) {
        runtimes.removeValue(forKey: path)
        recentProjects.removeAll { $0 == path }
        UserDefaults.standard.set(recentProjects, forKey: recentProjectsKey)
        if current.projectPath == path, let next = recentProjects.first {
            switchTo(next)
        }
    }

    private func runtime(for path: String) -> SessionViewModel {
        if let existing = runtimes[path] { return existing }
        let runtime = SessionViewModel(path: path)
        runtimes[path] = runtime
        wire(runtime)
        return runtime
    }

    private func remember(_ path: String) {
        guard !recentProjects.contains(path) else { return }
        recentProjects.insert(path, at: 0)
        recentProjects = Array(recentProjects.prefix(12))
        UserDefaults.standard.set(recentProjects, forKey: recentProjectsKey)
    }

    private func wire(_ runtime: SessionViewModel) {
        runtime.onWorkspaceMessage = { [weak self, weak runtime] target, task, id in
            guard let self, let source = runtime else { return }
            self.route(source: source, target: target, task: task, id: id)
        }
    }

    private func route(source: SessionViewModel, target: String, task: String, id: UInt64) {
        guard let path = recentProjects.first(where: { ($0 as NSString).lastPathComponent == target })
            ?? runtimes.keys.first(where: { ($0 as NSString).lastPathComponent == target }),
            FileManager.default.fileExists(atPath: path)
        else {
            source.respondWorkspaceMessage(id: id, reply: "(workspace '\(target)' is not available)")
            return
        }
        runtime(for: path).runExternalTask(task) { reply in
            source.respondWorkspaceMessage(id: id, reply: reply)
        }
    }
}
