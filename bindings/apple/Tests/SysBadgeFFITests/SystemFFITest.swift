//
//  File.swift
//  
//
//  Created by Finn Behrens on 29.10.23.
//

import XCTest
@testable import SysBadgeFFI

final class SystemFFITest: XCTestCase {
    func test_create() throws {
        let system = try SystemFFI("test")
        XCTAssertEqual(system.name(), "test")
    }
    
    func test_add_member() throws {
        let system = try SystemFFI("test")
        system.push_member(name: "Test Name", pronouns: "Test Pronouns")
        XCTAssertEqual(system.member_count, 1)
        let member = try system.member(0)
        XCTAssertEqual(member.name, "Test Name")
        XCTAssertEqual(member.pronouns, "Test Pronouns")
    }
    
    func test_from_system() throws {
        var system = System("Test System")
        system.append_member(name: "Test Member 1", pronouns: "he/him")
        system.append_member(name: "Test Member 3", pronouns: "they/them")
        system.append_member(name: "Test Member 2", pronouns: "she/her")
        XCTAssertEqual(system.members.count, 3)
        let system_ffi = try SystemFFI(from: system)
        system_ffi.sort()
        XCTAssertEqual(system_ffi.name(), "Test System")
        XCTAssertEqual(system_ffi.member_count, 3)
        
        var member: SystemFFI.Member
        member = try system_ffi.member(0)
        XCTAssertEqual(member.name, "Test Member 1")
        XCTAssertEqual(member.pronouns, "he/him")
        
        member = try system_ffi.member(1)
        XCTAssertEqual(member.name, "Test Member 2")
        XCTAssertEqual(member.pronouns, "she/her")
        
        member = try system_ffi.member(2)
        XCTAssertEqual(member.name, "Test Member 3")
        XCTAssertEqual(member.pronouns, "they/them")
    }
}
