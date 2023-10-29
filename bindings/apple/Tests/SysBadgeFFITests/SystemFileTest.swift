//
//  File.swift
//  
//
//  Created by Finn Behrens on 28.10.23.
//

import XCTest
@testable import SysBadgeFFI
import Foundation

let file = Bundle.module.path(forResource: "exmpl", ofType: "sysdf")!

final class SystemFileTest: XCTestCase {
    func test_file_open() throws {
        let file = try SystemFile(file: file)
        XCTAssertTrue(file.verify())
        XCTAssertEqual(file.name, "PluralKit Example System")
    }
    
    func test_read_system() throws {
        let file = try SystemFile(file: file)
        let system = try SystemFFI(from: file)
        XCTAssertEqual(system.member_count, 2)
        XCTAssertEqual(system.name(), "PluralKit Example System")
        XCTAssert(file.verify())
        
        let member = try system.member(0)
        XCTAssertEqual(member.name, "Myriad Kit")
        XCTAssertEqual(member.pronouns, "they/them")
    }
    
    func test_from_data() throws {
        let data = try Data(contentsOf: URL(fileURLWithPath: file))
        let file = try SystemFile(from: data)
        XCTAssert(file.verify())
        XCTAssertEqual(file.name, "PluralKit Example System")
    }
}
