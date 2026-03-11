import Foundation
import Network
import UIKit

@main
final class AppDelegate: UIResponder, UIApplicationDelegate {
    private lazy var driver: FrameDriver = {
        if let remote = RemoteDevDriver.makeFromEnvironment(appID: "album_list") {
            return remote
        }
        return LocalRustDriver()
    }()

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]? = nil
    ) -> Bool {
        driver.start()
        return true
    }
}

private protocol FrameDriver {
    func start()
}

private final class LocalRustDriver: FrameDriver {
    private var displayLink: CADisplayLink?
    private var lastBounds: CGRect = .zero
    private var lastInsets: UIEdgeInsets = .zero

    func start() {
        let bounds = UIScreen.main.bounds
        let insets = UIEdgeInsets.zero
        let started = mf_app_start(
            Float(bounds.width),
            Float(bounds.height),
            Float(insets.top),
            Float(insets.right),
            Float(insets.bottom),
            Float(insets.left)
        )
        assert(started, "Failed to start Rust example")
        installDisplayLink()
    }

    private func installDisplayLink() {
        guard displayLink == nil else {
            return
        }
        let displayLink = CADisplayLink(target: self, selector: #selector(onFrame))
        displayLink.add(to: .main, forMode: .common)
        self.displayLink = displayLink
    }

    @objc private func onFrame() {
        let metrics = currentMetrics()
        if metrics.bounds != lastBounds || metrics.insets != lastInsets {
            lastBounds = metrics.bounds
            lastInsets = metrics.insets
            mf_app_resize(
                Float(metrics.bounds.width),
                Float(metrics.bounds.height),
                Float(metrics.insets.top),
                Float(metrics.insets.right),
                Float(metrics.insets.bottom),
                Float(metrics.insets.left)
            )
        }
        mf_app_tick()
    }
}

private final class RemoteDevDriver: FrameDriver {
    private let appID: String
    private let connection: NWConnection
    private var displayLink: CADisplayLink?
    private var buffer = Data()
    private var lastBounds: CGRect = .zero
    private var lastInsets: UIEdgeInsets = .zero

    static func makeFromEnvironment(appID: String) -> RemoteDevDriver? {
        let environment = ProcessInfo.processInfo.environment
        guard let host = environment["MF_DEV_SERVER_HOST"] else {
            return nil
        }
        let portRaw = environment["MF_DEV_SERVER_PORT"] ?? "4488"
        guard let port = NWEndpoint.Port(portRaw) else {
            return nil
        }
        return RemoteDevDriver(appID: appID, connection: NWConnection(host: NWEndpoint.Host(host), port: port, using: .tcp))
    }

    init(appID: String, connection: NWConnection) {
        self.appID = appID
        self.connection = connection
    }

    func start() {
        connection.stateUpdateHandler = { [weak self] state in
            guard let self else { return }
            if case .ready = state {
                self.sendHello()
            }
        }
        connection.start(queue: .main)
        receiveLoop()
        installDisplayLink()
    }

    private func installDisplayLink() {
        guard displayLink == nil else {
            return
        }
        let displayLink = CADisplayLink(target: self, selector: #selector(onFrame))
        displayLink.add(to: .main, forMode: .common)
        self.displayLink = displayLink
    }

    private func receiveLoop() {
        connection.receive(minimumIncompleteLength: 1, maximumLength: 64 * 1024) { [weak self] data, _, isComplete, error in
            guard let self else { return }
            if let data, !data.isEmpty {
                self.buffer.append(data)
                self.flushServerLines()
            }
            if isComplete || error != nil {
                return
            }
            self.receiveLoop()
        }
    }

    private func flushServerLines() {
        while let newline = buffer.firstIndex(of: 0x0A) {
            let line = buffer[..<newline]
            buffer.removeSubrange(...newline)
            guard let payload = String(data: line, encoding: .utf8), !payload.isEmpty else {
                continue
            }
            payload.withCString { pointer in
                _ = mf_dev_renderer_apply_message(pointer)
            }
        }
    }

    @objc private func onFrame() {
        let metrics = currentMetrics()
        if metrics.bounds != lastBounds || metrics.insets != lastInsets {
            lastBounds = metrics.bounds
            lastInsets = metrics.insets
            sendHostResized(metrics: metrics)
        }
        forwardNativeEvents()
    }

    private func sendHello() {
        sendClientMessage(name: "Hello", body: [
            "app_id": appID,
            "host": hostDictionary(metrics: currentMetrics())
        ])
    }

    private func sendHostResized(metrics: (bounds: CGRect, insets: UIEdgeInsets)) {
        sendClientMessage(name: "HostResized", body: [
            "host": hostDictionary(metrics: metrics)
        ])
    }

    private func forwardNativeEvents() {
        guard let pointer = mf_dev_renderer_take_events_json() else {
            return
        }
        defer { mf_dev_renderer_clear_events_json() }
        let payload = String(cString: pointer)
        guard
            let data = payload.data(using: .utf8),
            let events = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]]
        else {
            return
        }

        for event in events {
            sendClientMessage(name: "UiEvent", body: event)
        }
    }

    private func sendClientMessage(name: String, body: [String: Any]) {
        guard
            let data = try? JSONSerialization.data(withJSONObject: [name: body], options: []),
            var payload = String(data: data, encoding: .utf8)
        else {
            return
        }
        payload.append("\n")
        connection.send(content: payload.data(using: .utf8), completion: .contentProcessed { _ in })
    }

    private func hostDictionary(metrics: (bounds: CGRect, insets: UIEdgeInsets)) -> [String: Any] {
        [
            "width": metrics.bounds.width,
            "height": metrics.bounds.height,
            "safe_area": [
                "top": metrics.insets.top,
                "right": metrics.insets.right,
                "bottom": metrics.insets.bottom,
                "left": metrics.insets.left
            ]
        ]
    }
}

private func currentMetrics() -> (bounds: CGRect, insets: UIEdgeInsets) {
    guard
        let windowScene = UIApplication.shared.connectedScenes
            .compactMap({ $0 as? UIWindowScene })
            .first,
        let window = windowScene.windows.first(where: \.isKeyWindow)
    else {
        return (UIScreen.main.bounds, .zero)
    }

    return (window.bounds, window.safeAreaInsets)
}
