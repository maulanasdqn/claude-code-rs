import SwiftUI

struct ThinkingView: View {
    let text: String
    @State private var expanded = true

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Button {
                withAnimation(.easeInOut(duration: 0.15)) { expanded.toggle() }
            } label: {
                HStack(spacing: 6) {
                    Image(systemName: "brain")
                    Text("Thinking")
                    Image(systemName: expanded ? "chevron.down" : "chevron.right")
                        .font(.caption2)
                }
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)

            if expanded {
                MarkdownText(raw: text)
                    .font(.callout)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .padding(10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color.secondary.opacity(0.06))
        .overlay(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .strokeBorder(Color.secondary.opacity(0.12))
        )
        .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
    }
}
