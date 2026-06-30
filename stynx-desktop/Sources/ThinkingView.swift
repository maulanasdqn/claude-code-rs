import SwiftUI

struct ThinkingView: View {
    let text: String
    @State private var enlarged = false

    private let collapsedHeight: CGFloat = 360

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 6) {
                Image(systemName: "brain")
                Text("Thinking")
                Spacer()
                Image(systemName: enlarged ? "chevron.up" : "chevron.down")
                    .font(.caption2)
            }
            .font(.caption.weight(.semibold))
            .foregroundStyle(.secondary)

            content
        }
        .padding(10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color.secondary.opacity(0.06))
        .overlay(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .strokeBorder(Color.secondary.opacity(0.12))
        )
        .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
        .contentShape(Rectangle())
        .onTapGesture { withAnimation(.easeInOut(duration: 0.18)) { enlarged.toggle() } }
    }

    @ViewBuilder
    private var content: some View {
        let body = MarkdownText(raw: text)
            .font(.callout)
            .foregroundStyle(.secondary)
            .frame(maxWidth: .infinity, alignment: .leading)

        if enlarged {
            body
        } else {
            body
                .frame(maxHeight: collapsedHeight, alignment: .top)
                .clipped()
                .mask(
                    LinearGradient(
                        stops: [
                            .init(color: .black, location: 0),
                            .init(color: .black, location: 0.82),
                            .init(color: .clear, location: 1),
                        ],
                        startPoint: .top,
                        endPoint: .bottom
                    )
                )
        }
    }
}
