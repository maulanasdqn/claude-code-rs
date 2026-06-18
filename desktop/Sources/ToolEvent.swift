import Foundation

enum PermissionModeOption: String, CaseIterable, Identifiable {
    case normal
    case auto
    case plan
    case bypass

    var id: String { rawValue }

    var label: String {
        switch self {
        case .normal: return "Normal"
        case .auto: return "Auto"
        case .plan: return "Plan"
        case .bypass: return "Bypass"
        }
    }

    var symbol: String {
        switch self {
        case .normal: return "checkmark.shield"
        case .auto: return "bolt"
        case .plan: return "list.bullet.clipboard"
        case .bypass: return "lock.open"
        }
    }
}
