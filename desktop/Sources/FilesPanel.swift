import SwiftUI

struct FilesPanel: View {
    @ObservedObject var model: SessionViewModel

    var body: some View {
        DiffListView(model: model)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
            .background(.ultraThinMaterial)
    }
}
