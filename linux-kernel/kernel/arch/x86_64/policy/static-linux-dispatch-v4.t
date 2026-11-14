use language (version is v0.1)
use library support (version is v0.1)
# TK-DEC-268; TOPAL-SYSTEMS-LINUX-DISPATCH-V4-001.
pub StaticLinuxOperation is Enum (ChangeDirectory, ChangeDirectoryByFile, ClockResolution, ClockSleep, ClockTime, Clone, Close, DirectoryEntries, Duplicate, DuplicateTo, DuplicateToFlags, EffectiveGroupId, EffectiveUserId, Execute, Exit, ExitGroup, FileControl, FileCreationMask, FileStatus, FileStatusAt, Fork, GroupId, IoControl, KernelIdentity, NanoSleep, Open, OpenAt, ParentId, Pipe, PipeFlags, Poll, ProcessControl, ProcessId, ProgramBreak, ProtectMemory, RandomBytes, Read, ReadAt, ReadLinkAt, RealtimeSeconds, RegisterRobustList, RegisterThreadId, ResourceLimit, ResourceLimits, RestartableSequences, Seek, SetGroupId, SetUserId, SignalAction, SignalMask, ThreadId, ThreadLocal, TruncateFile, Unlink, UnlinkAt, Unsupported, UserId, Wait, WorkingDirectory, Write, WriteAt, Yield)

select-range-0-0 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 0)
    true then Read
    false then Unsupported

select-range-1-1 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 1)
    true then Write
    false then Unsupported

select-range-2-2 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 2)
    true then Open
    false then Unsupported

select-range-1-2 is fn (number : Nat) -> StaticLinuxOperation
  number < 2
    true then select-range-1-1 number
    false then select-range-2-2 number

select-range-0-2 is fn (number : Nat) -> StaticLinuxOperation
  number < 1
    true then select-range-0-0 number
    false then select-range-1-2 number

select-range-3-3 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 3)
    true then Close
    false then Unsupported

select-range-5-5 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 5)
    true then FileStatus
    false then Unsupported

select-range-3-5 is fn (number : Nat) -> StaticLinuxOperation
  number < 5
    true then select-range-3-3 number
    false then select-range-5-5 number

select-range-7-7 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 7)
    true then Poll
    false then Unsupported

select-range-8-8 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 8)
    true then Seek
    false then Unsupported

select-range-7-8 is fn (number : Nat) -> StaticLinuxOperation
  number < 8
    true then select-range-7-7 number
    false then select-range-8-8 number

select-range-3-8 is fn (number : Nat) -> StaticLinuxOperation
  number < 7
    true then select-range-3-5 number
    false then select-range-7-8 number

select-range-0-8 is fn (number : Nat) -> StaticLinuxOperation
  number < 3
    true then select-range-0-2 number
    false then select-range-3-8 number

select-range-10-10 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 10)
    true then ProtectMemory
    false then Unsupported

select-range-12-12 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 12)
    true then ProgramBreak
    false then Unsupported

select-range-10-12 is fn (number : Nat) -> StaticLinuxOperation
  number < 12
    true then select-range-10-10 number
    false then select-range-12-12 number

select-range-13-13 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 13)
    true then SignalAction
    false then Unsupported

select-range-14-14 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 14)
    true then SignalMask
    false then Unsupported

select-range-13-14 is fn (number : Nat) -> StaticLinuxOperation
  number < 14
    true then select-range-13-13 number
    false then select-range-14-14 number

select-range-10-14 is fn (number : Nat) -> StaticLinuxOperation
  number < 13
    true then select-range-10-12 number
    false then select-range-13-14 number

select-range-16-16 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 16)
    true then IoControl
    false then Unsupported

select-range-17-17 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 17)
    true then ReadAt
    false then Unsupported

select-range-16-17 is fn (number : Nat) -> StaticLinuxOperation
  number < 17
    true then select-range-16-16 number
    false then select-range-17-17 number

select-range-18-18 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 18)
    true then WriteAt
    false then Unsupported

select-range-22-22 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 22)
    true then Pipe
    false then Unsupported

select-range-18-22 is fn (number : Nat) -> StaticLinuxOperation
  number < 22
    true then select-range-18-18 number
    false then select-range-22-22 number

select-range-16-22 is fn (number : Nat) -> StaticLinuxOperation
  number < 18
    true then select-range-16-17 number
    false then select-range-18-22 number

select-range-10-22 is fn (number : Nat) -> StaticLinuxOperation
  number < 16
    true then select-range-10-14 number
    false then select-range-16-22 number

select-range-0-22 is fn (number : Nat) -> StaticLinuxOperation
  number < 10
    true then select-range-0-8 number
    false then select-range-10-22 number

select-range-24-24 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 24)
    true then Yield
    false then Unsupported

select-range-32-32 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 32)
    true then Duplicate
    false then Unsupported

select-range-33-33 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 33)
    true then DuplicateTo
    false then Unsupported

select-range-32-33 is fn (number : Nat) -> StaticLinuxOperation
  number < 33
    true then select-range-32-32 number
    false then select-range-33-33 number

select-range-24-33 is fn (number : Nat) -> StaticLinuxOperation
  number < 32
    true then select-range-24-24 number
    false then select-range-32-33 number

select-range-35-35 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 35)
    true then NanoSleep
    false then Unsupported

select-range-39-39 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 39)
    true then ProcessId
    false then Unsupported

select-range-35-39 is fn (number : Nat) -> StaticLinuxOperation
  number < 39
    true then select-range-35-35 number
    false then select-range-39-39 number

select-range-56-56 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 56)
    true then Clone
    false then Unsupported

select-range-57-57 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 57)
    true then Fork
    false then Unsupported

select-range-56-57 is fn (number : Nat) -> StaticLinuxOperation
  number < 57
    true then select-range-56-56 number
    false then select-range-57-57 number

select-range-35-57 is fn (number : Nat) -> StaticLinuxOperation
  number < 56
    true then select-range-35-39 number
    false then select-range-56-57 number

select-range-24-57 is fn (number : Nat) -> StaticLinuxOperation
  number < 35
    true then select-range-24-33 number
    false then select-range-35-57 number

select-range-59-59 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 59)
    true then Execute
    false then Unsupported

select-range-60-60 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 60)
    true then Exit
    false then Unsupported

select-range-59-60 is fn (number : Nat) -> StaticLinuxOperation
  number < 60
    true then select-range-59-59 number
    false then select-range-60-60 number

select-range-61-61 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 61)
    true then Wait
    false then Unsupported

select-range-63-63 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 63)
    true then KernelIdentity
    false then Unsupported

select-range-61-63 is fn (number : Nat) -> StaticLinuxOperation
  number < 63
    true then select-range-61-61 number
    false then select-range-63-63 number

select-range-59-63 is fn (number : Nat) -> StaticLinuxOperation
  number < 61
    true then select-range-59-60 number
    false then select-range-61-63 number

select-range-72-72 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 72)
    true then FileControl
    false then Unsupported

select-range-77-77 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 77)
    true then TruncateFile
    false then Unsupported

select-range-72-77 is fn (number : Nat) -> StaticLinuxOperation
  number < 77
    true then select-range-72-72 number
    false then select-range-77-77 number

select-range-79-79 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 79)
    true then WorkingDirectory
    false then Unsupported

select-range-80-80 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 80)
    true then ChangeDirectory
    false then Unsupported

select-range-79-80 is fn (number : Nat) -> StaticLinuxOperation
  number < 80
    true then select-range-79-79 number
    false then select-range-80-80 number

select-range-72-80 is fn (number : Nat) -> StaticLinuxOperation
  number < 79
    true then select-range-72-77 number
    false then select-range-79-80 number

select-range-59-80 is fn (number : Nat) -> StaticLinuxOperation
  number < 72
    true then select-range-59-63 number
    false then select-range-72-80 number

select-range-24-80 is fn (number : Nat) -> StaticLinuxOperation
  number < 59
    true then select-range-24-57 number
    false then select-range-59-80 number

select-range-0-80 is fn (number : Nat) -> StaticLinuxOperation
  number < 24
    true then select-range-0-22 number
    false then select-range-24-80 number

select-range-81-81 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 81)
    true then ChangeDirectoryByFile
    false then Unsupported

select-range-87-87 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 87)
    true then Unlink
    false then Unsupported

select-range-95-95 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 95)
    true then FileCreationMask
    false then Unsupported

select-range-87-95 is fn (number : Nat) -> StaticLinuxOperation
  number < 95
    true then select-range-87-87 number
    false then select-range-95-95 number

select-range-81-95 is fn (number : Nat) -> StaticLinuxOperation
  number < 87
    true then select-range-81-81 number
    false then select-range-87-95 number

select-range-97-97 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 97)
    true then ResourceLimit
    false then Unsupported

select-range-102-102 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 102)
    true then UserId
    false then Unsupported

select-range-97-102 is fn (number : Nat) -> StaticLinuxOperation
  number < 102
    true then select-range-97-97 number
    false then select-range-102-102 number

select-range-104-104 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 104)
    true then GroupId
    false then Unsupported

select-range-105-105 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 105)
    true then SetUserId
    false then Unsupported

select-range-104-105 is fn (number : Nat) -> StaticLinuxOperation
  number < 105
    true then select-range-104-104 number
    false then select-range-105-105 number

select-range-97-105 is fn (number : Nat) -> StaticLinuxOperation
  number < 104
    true then select-range-97-102 number
    false then select-range-104-105 number

select-range-81-105 is fn (number : Nat) -> StaticLinuxOperation
  number < 97
    true then select-range-81-95 number
    false then select-range-97-105 number

select-range-106-106 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 106)
    true then SetGroupId
    false then Unsupported

select-range-107-107 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 107)
    true then EffectiveUserId
    false then Unsupported

select-range-106-107 is fn (number : Nat) -> StaticLinuxOperation
  number < 107
    true then select-range-106-106 number
    false then select-range-107-107 number

select-range-108-108 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 108)
    true then EffectiveGroupId
    false then Unsupported

select-range-110-110 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 110)
    true then ParentId
    false then Unsupported

select-range-108-110 is fn (number : Nat) -> StaticLinuxOperation
  number < 110
    true then select-range-108-108 number
    false then select-range-110-110 number

select-range-106-110 is fn (number : Nat) -> StaticLinuxOperation
  number < 108
    true then select-range-106-107 number
    false then select-range-108-110 number

select-range-157-157 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 157)
    true then ProcessControl
    false then Unsupported

select-range-158-158 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 158)
    true then ThreadLocal
    false then Unsupported

select-range-157-158 is fn (number : Nat) -> StaticLinuxOperation
  number < 158
    true then select-range-157-157 number
    false then select-range-158-158 number

select-range-186-186 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 186)
    true then ThreadId
    false then Unsupported

select-range-201-201 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 201)
    true then RealtimeSeconds
    false then Unsupported

select-range-186-201 is fn (number : Nat) -> StaticLinuxOperation
  number < 201
    true then select-range-186-186 number
    false then select-range-201-201 number

select-range-157-201 is fn (number : Nat) -> StaticLinuxOperation
  number < 186
    true then select-range-157-158 number
    false then select-range-186-201 number

select-range-106-201 is fn (number : Nat) -> StaticLinuxOperation
  number < 157
    true then select-range-106-110 number
    false then select-range-157-201 number

select-range-81-201 is fn (number : Nat) -> StaticLinuxOperation
  number < 106
    true then select-range-81-105 number
    false then select-range-106-201 number

select-range-217-217 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 217)
    true then DirectoryEntries
    false then Unsupported

select-range-218-218 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 218)
    true then RegisterThreadId
    false then Unsupported

select-range-217-218 is fn (number : Nat) -> StaticLinuxOperation
  number < 218
    true then select-range-217-217 number
    false then select-range-218-218 number

select-range-228-228 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 228)
    true then ClockTime
    false then Unsupported

select-range-229-229 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 229)
    true then ClockResolution
    false then Unsupported

select-range-228-229 is fn (number : Nat) -> StaticLinuxOperation
  number < 229
    true then select-range-228-228 number
    false then select-range-229-229 number

select-range-217-229 is fn (number : Nat) -> StaticLinuxOperation
  number < 228
    true then select-range-217-218 number
    false then select-range-228-229 number

select-range-230-230 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 230)
    true then ClockSleep
    false then Unsupported

select-range-231-231 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 231)
    true then ExitGroup
    false then Unsupported

select-range-230-231 is fn (number : Nat) -> StaticLinuxOperation
  number < 231
    true then select-range-230-230 number
    false then select-range-231-231 number

select-range-257-257 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 257)
    true then OpenAt
    false then Unsupported

select-range-262-262 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 262)
    true then FileStatusAt
    false then Unsupported

select-range-257-262 is fn (number : Nat) -> StaticLinuxOperation
  number < 262
    true then select-range-257-257 number
    false then select-range-262-262 number

select-range-230-262 is fn (number : Nat) -> StaticLinuxOperation
  number < 257
    true then select-range-230-231 number
    false then select-range-257-262 number

select-range-217-262 is fn (number : Nat) -> StaticLinuxOperation
  number < 230
    true then select-range-217-229 number
    false then select-range-230-262 number

select-range-263-263 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 263)
    true then UnlinkAt
    false then Unsupported

select-range-267-267 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 267)
    true then ReadLinkAt
    false then Unsupported

select-range-263-267 is fn (number : Nat) -> StaticLinuxOperation
  number < 267
    true then select-range-263-263 number
    false then select-range-267-267 number

select-range-273-273 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 273)
    true then RegisterRobustList
    false then Unsupported

select-range-292-292 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 292)
    true then DuplicateToFlags
    false then Unsupported

select-range-273-292 is fn (number : Nat) -> StaticLinuxOperation
  number < 292
    true then select-range-273-273 number
    false then select-range-292-292 number

select-range-263-292 is fn (number : Nat) -> StaticLinuxOperation
  number < 273
    true then select-range-263-267 number
    false then select-range-273-292 number

select-range-293-293 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 293)
    true then PipeFlags
    false then Unsupported

select-range-302-302 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 302)
    true then ResourceLimits
    false then Unsupported

select-range-293-302 is fn (number : Nat) -> StaticLinuxOperation
  number < 302
    true then select-range-293-293 number
    false then select-range-302-302 number

select-range-318-318 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 318)
    true then RandomBytes
    false then Unsupported

select-range-334-334 is fn (number : Nat) -> StaticLinuxOperation
  support syscall matches-selector? (number, 334)
    true then RestartableSequences
    false then Unsupported

select-range-318-334 is fn (number : Nat) -> StaticLinuxOperation
  number < 334
    true then select-range-318-318 number
    false then select-range-334-334 number

select-range-293-334 is fn (number : Nat) -> StaticLinuxOperation
  number < 318
    true then select-range-293-302 number
    false then select-range-318-334 number

select-range-263-334 is fn (number : Nat) -> StaticLinuxOperation
  number < 293
    true then select-range-263-292 number
    false then select-range-293-334 number

select-range-217-334 is fn (number : Nat) -> StaticLinuxOperation
  number < 263
    true then select-range-217-262 number
    false then select-range-263-334 number

select-range-81-334 is fn (number : Nat) -> StaticLinuxOperation
  number < 217
    true then select-range-81-201 number
    false then select-range-217-334 number

select-range-0-334 is fn (number : Nat) -> StaticLinuxOperation
  number < 81
    true then select-range-0-80 number
    false then select-range-81-334 number

pub select-operation is fn (number : Nat) -> StaticLinuxOperation
  select-range-0-334 number
