import UIKit

@main
final class AppDelegate: UIResponder, UIApplicationDelegate {
    private let driver = RustExampleDriver()

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]? = nil
    ) -> Bool {
        let bounds = UIScreen.main.bounds
        let started = mf_examples_start(
            SelectedExample.current.rawValue,
            Float(bounds.width),
            Float(bounds.height)
        )
        assert(started, "Failed to start Rust example")
        driver.start()
        return true
    }
}

private final class RustExampleDriver {
    private var displayLink: CADisplayLink?
    private var lastBounds: CGRect = .zero

    func start() {
        guard displayLink == nil else {
            return
        }
        let displayLink = CADisplayLink(target: self, selector: #selector(onFrame))
        displayLink.add(to: .main, forMode: .common)
        self.displayLink = displayLink
    }

    @objc private func onFrame() {
        let bounds = UIScreen.main.bounds
        if bounds != lastBounds {
            lastBounds = bounds
            mf_examples_resize(Float(bounds.width), Float(bounds.height))
        }
        mf_examples_tick()
    }
}
