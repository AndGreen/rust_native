import Foundation

enum SelectedExample: UInt32 {
    case counter = 1
    case albumList = 2

    static let current: SelectedExample = .albumList
}
