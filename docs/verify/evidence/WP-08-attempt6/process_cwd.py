import ctypes,struct
def process_cwd(pid):
 # Read only the CurrentDirectory UNICODE_STRING of our identified x64 child.
 k=ctypes.WinDLL('kernel32',use_last_error=True);n=ctypes.WinDLL('ntdll')
 k.OpenProcess.argtypes=[ctypes.c_uint32,ctypes.c_bool,ctypes.c_uint32];k.OpenProcess.restype=ctypes.c_void_p
 k.ReadProcessMemory.argtypes=[ctypes.c_void_p,ctypes.c_void_p,ctypes.c_void_p,ctypes.c_size_t,ctypes.POINTER(ctypes.c_size_t)]
 k.CloseHandle.argtypes=[ctypes.c_void_p]
 n.NtQueryInformationProcess.argtypes=[ctypes.c_void_p,ctypes.c_uint32,ctypes.c_void_p,ctypes.c_uint32,ctypes.POINTER(ctypes.c_uint32)]
 handle=k.OpenProcess(0x410,False,pid);assert handle,ctypes.get_last_error()
 try:
  info=ctypes.create_string_buffer(48);size=ctypes.c_uint32();assert n.NtQueryInformationProcess(handle,0,info,48,ctypes.byref(size))==0
  def read(ptr,size):
   data=ctypes.create_string_buffer(size);done=ctypes.c_size_t();assert k.ReadProcessMemory(handle,ptr,data,size,ctypes.byref(done)) and done.value==size;return data.raw
  peb=struct.unpack_from('<Q',info.raw,8)[0];params=struct.unpack('<Q',read(peb+0x20,8))[0];u=read(params+0x38,16);length=struct.unpack_from('<H',u)[0];buffer=struct.unpack_from('<Q',u,8)[0];return read(buffer,length).decode('utf-16le')
 finally:k.CloseHandle(handle)
if __name__=='__main__':
 import sys
 print(process_cwd(int(sys.argv[1])))
