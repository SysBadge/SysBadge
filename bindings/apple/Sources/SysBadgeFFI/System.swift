//
//  File.swift
//  
//
//  Created by Finn Behrens on 29.10.23.
//

import Foundation

public struct System: Codable {
    public var name: String
    public var members: [Member] = []
   
    /// Create new system with name.
    public init(_ name: String) {
        self.name = name
    }
   
    /// Create system from a FFI system.
    public init(from system_ffi: SystemFFI) throws {
        self.name = system_ffi.name()
        
        for i in 0..<system_ffi.member_count {
            self.members.append(try system_ffi.member(Int(i)))
        }
    }
    
}

extension System {
    public struct Member: Codable {
        public var name: String
        public var pronouns: String
        
        public init(name: String, pronouns: String = "") {
            self.name = name
            self.pronouns = pronouns
        }
    }
}
