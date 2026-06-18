import SwiftUI

@main
struct StynxApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
                .frame(minWidth: 860, minHeight: 560)
        }
        .windowToolbarStyle(.unified(showsTitle: true))
        .windowStyle(.titleBar)
    }
}
