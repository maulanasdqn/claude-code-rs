import SwiftUI

struct TypingDots: View {
    @State private var phase = 0.0

    var body: some View {
        HStack(spacing: 5) {
            ForEach(0..<3, id: \.self) { index in
                Circle()
                    .fill(.secondary)
                    .frame(width: 7, height: 7)
                    .opacity(opacity(for: index))
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
        .background(Color.primary.opacity(0.06))
        .clipShape(Capsule())
        .onAppear {
            withAnimation(.easeInOut(duration: 0.9).repeatForever(autoreverses: true)) {
                phase = 1.0
            }
        }
    }

    private func opacity(for index: Int) -> Double {
        let base = 0.3 + 0.7 * phase
        let shift = Double(index) * 0.25
        let value = base - shift
        return max(0.25, min(1.0, value))
    }
}
