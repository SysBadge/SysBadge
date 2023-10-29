//
//  File.swift
//  
//
//  Created by Finn Behrens on 28.10.23.
//

#if canImport(Darwin)
import Darwin
#else
import Glibc
#endif

#if canImport(Foundation)
import Foundation
#endif

import sysbadge_ffi

public class SystemFile {
    var sb_file: OpaquePointer? // sb_file
    var initialized: Bool = false
    
    public init(file: String) throws {
        let err = file.withCString { file in
            sysbadge_ffi.sb_file_open(file, &self.sb_file)
        }
        if err < 0 {
            throw ErrorFFI(err_code: err)
        }
        self.initialized = true
    }
    
    public init(from bytes: [UInt8]) throws {
        let err = bytes.withUnsafeBufferPointer { ptr in
            sysbadge_ffi.sb_file_open_bytes(&self.sb_file, ptr.baseAddress, ptr.count)
        }
        if err < 0 {
            throw ErrorFFI(err_code: err)
        }
        self.initialized = true
    }
    
    #if canImport(Foundation)
    public init(from data: Data) throws {
        let err = data.withUnsafeBytes { ptr in
            sysbadge_ffi.sb_file_open_bytes(&self.sb_file, ptr.baseAddress, ptr.count)
        }
        if err < 0 {
            throw ErrorFFI(err_code: err)
        }
        self.initialized = true
    }
    #endif
   
    public var name: String {
        let name = sysbadge_ffi.sb_file_system_name(self.sb_file)
        return String(cString: name!)
    }
    
    public func json() -> String {
        let json = sysbadge_ffi.sb_file_json(self.sb_file)
        let string = String(cString: json!)
        sysbadge_ffi.sb_free_string(json)
        return string
    }
    
    public func verify() -> Bool {
        sysbadge_ffi.sb_file_verify(self.sb_file)
    }
    
    deinit {
        if self.initialized {
            sysbadge_ffi.sb_file_free(self.sb_file)
        }
    }
}
