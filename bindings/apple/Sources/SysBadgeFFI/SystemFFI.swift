//
//  File.swift
//  
//
//  Created by Finn Behrens on 29.10.23.
//

#if canImport(Darwin)
import Darwin
#else
import Glibc
#endif
import sysbadge_ffi

/// System managed on the rust side.
public class SystemFFI {
    var sb_system: OpaquePointer?
    
    /// Create a new System with name.
    public init(_ name: String) throws {
        let err = name.withCString { name in
            sysbadge_ffi.sb_system_new(&self.sb_system, name)
        }
        if err != 0 {
            throw ErrorFFI(err_code: err)
        }
    }
    
    /// Convert a file into a system.
    public init(from file: SystemFile) throws {
        let err = sysbadge_ffi.sb_file_system(file.sb_file, &self.sb_system)
        if err < 0 {
            throw ErrorFFI(err_code: err)
        }
    }
    
    /// Convert a swift owned system into a rust owned system.
    public convenience init(from system: System) throws {
        try self.init(system.name)
        for member in system.members {
            self.push_member(member)
        }
    }
    
    deinit {
        sysbadge_ffi.sb_system_free(self.sb_system)
    }
    
    /// Return name stored in the System.
    public func name() -> String {
        let name = sysbadge_ffi.sb_system_name(self.sb_system)
        let string = String(cString: name!)
        sysbadge_ffi.sb_free_string(name)
        return string
    }
    
    /// Member count.
    var member_count: UInt {
        UInt(sysbadge_ffi.sb_system_member_count(self.sb_system))
    }
    
    /// Get member with the specified index.
    public func member(_ index: Int) throws -> Member {
        var member: sysbadge_ffi.sb_system_member = sysbadge_ffi.sb_system_member()
        let ret = sysbadge_ffi.sb_system_get_member(self.sb_system, index, &member)
        if ret != 0 {
            throw ErrorFFI(err_code: ret)
        }
        
        let name = String(cString: member.name)
        let pronouns = String(cString: member.pronouns)
       
        // free member as copied to own string representation
        sysbadge_ffi.sb_system_member_free(&member)
        
        return Member(name: name, pronouns: pronouns)
    }
    
    /// Add member to the system.
    public func push_member(_ member: Member) {
        var cmember: sysbadge_ffi.sb_system_member = sysbadge_ffi.sb_system_member()
        member.name.withCString { name in
            cmember.name = name
        }
        member.pronouns.withCString { pronouns in
            cmember.pronouns = pronouns
        }
        sysbadge_ffi.sb_system_push_member(self.sb_system, &cmember)
    }
    
    /// Sort system mebers.
    public func sort() {
        sysbadge_ffi.sb_system_sort(self.sb_system)
    }
    
    public typealias Member = System.Member
}
