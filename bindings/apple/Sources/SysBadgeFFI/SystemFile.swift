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
import sysbadge_ffi

public class SystemFile {
    var sb_file: OpaquePointer? // sb_file
    
    public init(file: String) throws {
        let err = file.withCString { file in
            sysbadge_ffi.sb_file_open(file, &self.sb_file)
        }
        if err != 0 {
            throw ErrorFFI(err_code: err)
        }
    }
   
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
        sysbadge_ffi.sb_file_free(self.sb_file)
    }
}
