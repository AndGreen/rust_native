import UIKit

@main
final class AppDelegate: UIResponder, UIApplicationDelegate {
    private let driver = RustExampleDriver()

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]? = nil
    ) -> Bool {
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
        driver.start()
        return true
    }
}

private final class RustExampleDriver {
    private var displayLink: CADisplayLink?
    private var lastBounds: CGRect = .zero
    private var lastInsets: UIEdgeInsets = .zero

    func start() {
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
}
