import Foundation

enum DiffKind {
    case context
    case added
    case removed
}

struct DiffLine: Identifiable, Equatable {
    let id = UUID()
    let kind: DiffKind
    let text: String
    let oldNumber: Int?
    let newNumber: Int?
}

func computeDiff(old: [String], new: [String]) -> [DiffLine] {
    let n = old.count
    let m = new.count
    var lcs = Array(repeating: Array(repeating: 0, count: m + 1), count: n + 1)
    if n > 0 && m > 0 {
        for i in stride(from: n - 1, through: 0, by: -1) {
            for j in stride(from: m - 1, through: 0, by: -1) {
                lcs[i][j] = old[i] == new[j]
                    ? lcs[i + 1][j + 1] + 1
                    : max(lcs[i + 1][j], lcs[i][j + 1])
            }
        }
    }

    var result: [DiffLine] = []
    var i = 0
    var j = 0
    var oldNum = 1
    var newNum = 1

    while i < n, j < m {
        if old[i] == new[j] {
            result.append(DiffLine(kind: .context, text: old[i], oldNumber: oldNum, newNumber: newNum))
            i += 1; j += 1; oldNum += 1; newNum += 1
        } else if lcs[i + 1][j] >= lcs[i][j + 1] {
            result.append(DiffLine(kind: .removed, text: old[i], oldNumber: oldNum, newNumber: nil))
            i += 1; oldNum += 1
        } else {
            result.append(DiffLine(kind: .added, text: new[j], oldNumber: nil, newNumber: newNum))
            j += 1; newNum += 1
        }
    }
    while i < n {
        result.append(DiffLine(kind: .removed, text: old[i], oldNumber: oldNum, newNumber: nil))
        i += 1; oldNum += 1
    }
    while j < m {
        result.append(DiffLine(kind: .added, text: new[j], oldNumber: nil, newNumber: newNum))
        j += 1; newNum += 1
    }
    return result
}

struct FileChange: Identifiable, Equatable {
    let id = UUID()
    let path: String
    let kind: String
    var diff: [DiffLine]?
    var lines: [String]?
    var adds: Int = 0
    var removes: Int = 0

    var name: String { (path as NSString).lastPathComponent }
    var badge: String { kind == "file_edit" ? "M" : "A" }
}

func parseToolArgs(_ json: String) -> [String: Any]? {
    guard let data = json.data(using: .utf8) else { return nil }
    return (try? JSONSerialization.jsonObject(with: data)) as? [String: Any]
}
