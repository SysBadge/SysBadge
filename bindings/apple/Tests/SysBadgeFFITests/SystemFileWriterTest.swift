//
//  SystemFileWriterTest.swift
//  
//
//  Created by Finn Behrens on 29.10.23.
//

import XCTest
@testable import SysBadgeFFI

final class SystemFileWriterTest: XCTestCase {
    func test_write_data() throws {
        let system = try SystemFFI("Test System")
        system.push_member(name: "Test Member", pronouns: "Test Pronouns")
        let writer = SystemFileWriter(from: system)
        writer.flags = [.checksum]
        
        let data = writer.data()
        let file = try SystemFile(from: data)
        XCTAssertEqual(file.name, "Test System")
    }
}
