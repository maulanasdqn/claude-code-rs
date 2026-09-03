import Foundation

final class EventBridge: EventListener {
    private let forward: (FfiEvent) -> Void

    init(_ forward: @escaping (FfiEvent) -> Void) {
        self.forward = forward
    }

    func onEvent(event: FfiEvent) {
        forward(event)
    }
}
