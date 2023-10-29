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

struct ErrorFFI {
    let code: sysbadge_ffi.sb_status_code
    
    init(err_code: Int32) {
        self.code = sysbadge_ffi.sb_status_code(UInt32(-err_code))
    }
}

extension ErrorFFI: Error {}

extension ErrorFFI: CustomDebugStringConvertible {
    var debugDescription: String {
        let sb = sysbadge_ffi.sb_status_code_debug_string(self.code)
        let str = String(cString: sb!)
        sysbadge_ffi.sb_free_string(sb)
        return str
    }
}

extension ErrorFFI: CustomStringConvertible {
    var description: String {
        let sb = sysbadge_ffi.sb_status_code_string(self.code)
        let str = String(cString: sb!)
        sysbadge_ffi.sb_free_string(sb)
        return str
    }
}

