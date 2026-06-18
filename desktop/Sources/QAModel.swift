import Foundation

struct QAOption: Decodable, Identifiable {
    let label: String
    let description: String?
    var id: String { label }
}

struct QAQuestion: Decodable, Identifiable {
    let question: String
    let header: String
    let multiSelect: Bool?
    let options: [QAOption]
    let id = UUID()

    private enum CodingKeys: String, CodingKey {
        case question, header, multiSelect, options
    }

    var allowsMultiple: Bool { multiSelect ?? false }
}

private struct QAEnvelope: Decodable {
    let questions: [QAQuestion]
}

func parseQA(_ raw: String) -> [QAQuestion]? {
    let marker = "@@QA@@"
    guard raw.hasPrefix(marker) else { return nil }
    let json = String(raw.dropFirst(marker.count))
    guard let data = json.data(using: .utf8),
          let envelope = try? JSONDecoder().decode(QAEnvelope.self, from: data) else {
        return nil
    }
    return envelope.questions
}
