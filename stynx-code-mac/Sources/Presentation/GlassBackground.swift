import SwiftUI

extension View {
    @ViewBuilder
    func glassBackground(cornerRadius: CGFloat) -> some View {
        let shape = RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
        if #available(macOS 26.0, *) {
            self.glassEffect(.regular.interactive(), in: shape)
        } else {
            self
                .background(.ultraThinMaterial, in: shape)
                .overlay(shape.strokeBorder(Color.white.opacity(0.10)))
                .shadow(color: .black.opacity(0.18), radius: 8, y: 2)
        }
    }
}
