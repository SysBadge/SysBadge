// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "SysBadgeFFI",
    platforms: [
        .macOS("11.0"),
        .iOS("14.0"),
    ],
    products: [
        // Products define the executables and libraries a package produces, making them visible to other packages.
        .library(
            name: "SysBadgeFFI",
            targets: ["SysBadgeFFI"]),
    ],
    targets: [
        // Targets are the basic building blocks of a package, defining a module or a test suite.
        // Targets can depend on other targets in this package and products from dependencies.
        .systemLibrary(name: "sysbadge_ffi"),
        .target(
            name: "SysBadgeFFI",
            dependencies: ["sysbadge_ffi", ],
            swiftSettings: [.unsafeFlags(["-I", "./generated"])],
            linkerSettings: [
                .unsafeFlags(["-L./generated"]),
                .linkedFramework("SystemConfiguration")
            ]
        ),
            // swiftSettings: [.unsafeFlags(["-L", "../../target/debug"])]),
        .testTarget(
            name: "SysBadgeFFITests",
            dependencies: ["SysBadgeFFI"],
            resources: [.copy("exmpl.sysdf")]),
    ],
    swiftLanguageVersions: [.v5]
)
