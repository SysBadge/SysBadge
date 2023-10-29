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

public class SystemFileWriter {
    var writer: sysbadge_ffi.sb_file_writer
    public let system: SystemFFI // keep reference as writer holds one and we cannot uninit before.
    
    public init(from system: SystemFFI)  {
        self.writer = sysbadge_ffi.sb_system_file_writer_new(system.sb_system)
        self.system = system
    }
    
    public convenience init(from system: System) throws {
        let system = try SystemFFI(from: system)
        self.init(from: system)
    }
    
    public convenience init(from file: SystemFile) throws {
        let system = try SystemFFI(from: file)
        self.init(from: system)
    }
   
    public var flags: Flags {
        get {
            Flags(rawValue: self.writer.flags)
        }
        set {
            self.writer.flags = newValue.rawValue
        }
    }
    
    public func write(file: String) throws {
        let err = file.withCString { file in
            sysbadge_ffi.sb_system_file_writer_write(&self.writer, file)
        }
        if err < 0 {
            throw ErrorFFI(err_code: err)
        }
    }
    
    public func bytes() throws -> [UInt8] {
        var ptr: UnsafeMutablePointer<UInt8>?
        let size = sysbadge_ffi.sb_system_file_writer_bytes(&self.writer, &ptr)
        
        let arr = Array(UnsafeBufferPointer(start: ptr, count: size))
        sysbadge_ffi.sb_free_buffer(ptr!, size)
        return arr
    }
    
    public struct Flags: OptionSet {
        public var rawValue: UInt32
        
        public init(rawValue: UInt32) {
            self.rawValue = rawValue
        }
        
        static let checksum = Flags(rawValue: sysbadge_ffi.sb_file_flags_SHA2_CHECKSUM)
        static let jsonBlob = Flags(rawValue: sysbadge_ffi.sb_file_flags_JSON_BLOB)
        
        static let all: Flags = [.checksum, .jsonBlob]
    }
}
