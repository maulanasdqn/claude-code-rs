import SwiftUI

struct CrossWorkspaceCard: View {
    let tool: ToolItem
    let incoming: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 8) {
                Image(systemName: incoming ? "tray.and.arrow.down.fill" : "paperplane.fill")
                    .font(.system(size: 12, weight: .semibold))
                    .foregroundStyle(Color.accentColor)
                    .frame(width: 24, height: 24)
                    .background(Color.accentColor.opacity(0.15))
                    .clipShape(RoundedRectangle(cornerRadius: 7, style: .continuous))

                if incoming {
                    Text("Incoming from another workspace")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.secondary)
                } else {
                    HStack(spacing: 6) {
                        chip("this workspace")
                        Image(systemName: "arrow.right").font(.caption2).foregroundStyle(.secondary)
                        chip(tool.title.isEmpty ? "workspace" : tool.title)
                    }
                }

                Spacer(minLength: 6)
                status
            }

            if let task = tool.subtitle, !task.isEmpty {
                Text(task)
                    .font(.callout)
                    .foregroundStyle(.primary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }

            if !incoming, !tool.running, !tool.detail.isEmpty {
                Text("↩ " + tool.detail.prefix(400))
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .glassBackground(cornerRadius: 12)
        .overlay(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .strokeBorder(Color.accentColor.opacity(0.3), lineWidth: 1)
        )
        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private func chip(_ text: String) -> some View {
        Text(text)
            .font(.caption.weight(.medium))
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(Color.primary.opacity(0.06))
            .clipShape(Capsule())
    }

    @ViewBuilder
    private var status: some View {
        if tool.running {
            HStack(spacing: 5) {
                ProgressView().controlSize(.small).scaleEffect(0.7)
                Text(incoming ? "working…" : "waiting…")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
        } else if tool.isError {
            Image(systemName: "exclamationmark.triangle.fill").foregroundStyle(.orange).font(.caption)
        } else {
            Image(systemName: "checkmark.circle.fill").foregroundStyle(.green).font(.caption)
        }
    }
}
