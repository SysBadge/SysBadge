//
//  File.swift
//  
//
//  Created by Finn Behrens on 28.10.23.
//

import XCTest
@testable import SysBadgeFFI

final class SystemFileTest: XCTestCase {
    func test_file_open() throws {
        let file = try SystemFile(file: "../../tests/exmpl.sysdf")
        XCTAssertTrue(file.verify())
        XCTAssertEqual(file.name, "PluralKit Example System")
    }
    
    func test_read_system() throws {
        let file = try SystemFile(file: "../../tests/exmpl.sysdf")
        let system = try SystemFFI(from: file)
        XCTAssertEqual(system.member_count, 2)
        XCTAssertEqual(system.name(), "PluralKit Example System")
        
        let member = try system.member(0)
        XCTAssertEqual(member.name, "Myriad Kit")
        XCTAssertEqual(member.pronouns, "they/them")
    }
}
