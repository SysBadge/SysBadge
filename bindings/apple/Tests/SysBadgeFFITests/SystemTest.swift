//
//  SystemTest.swift
//  
//
//  Created by Finn Behrens on 29.10.23.
//

import XCTest
@testable import SysBadgeFFI

final class SystemTest: XCTestCase {
    func test_from_ffi() throws {
        let system_ffi = try SystemFFI("test")
        system_ffi.push_member(name: "Test Name", pronouns: "Test Pronouns")
        XCTAssertEqual(system_ffi.member_count, 1)
        
        let system = try System(from: system_ffi)
        XCTAssertEqual(system.name, "test")
        XCTAssertEqual(system.members[0].name, "Test Name")
        XCTAssertEqual(system.members[0].pronouns, "Test Pronouns")
    }
}
