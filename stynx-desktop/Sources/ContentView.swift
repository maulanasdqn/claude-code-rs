import SwiftUI

struct ContentView: View {
    @StateObject private var app = AppModel()
    @State private var columnVisibility: NavigationSplitViewVisibility = .all

    var body: some View {
        NavigationSplitView(columnVisibility: $columnVisibility) {
            SidebarView(app: app, model: app.current)
                .navigationSplitViewColumnWidth(min: 220, ideal: 260, max: 320)
        } detail: {
            WorkspaceView(model: app.current)
                .id(app.current.projectPath)
        }
    }
}
