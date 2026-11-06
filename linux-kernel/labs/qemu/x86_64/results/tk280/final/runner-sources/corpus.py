import importlib.util,tempfile
from pathlib import Path
spec=importlib.util.spec_from_file_location("gate","linux-kernel/labs/qemu/x86_64/topal_gate.py")
g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
clean=g.verify(30,True)
print("PASS clean",clean["linked_kernel_sha256"],flush=True)
external=Path("/tmp/topal-busybox-elf.myVNMa/busybox")
with tempfile.TemporaryDirectory(prefix="topal-corpus-",dir="/tmp") as temporary:
 values=g.build_image(Path(temporary)/"artifact",external)
 original=g.build_image
 g.build_image=lambda *_a,**_k:values
 try:
  for workload in g.PROCESS_WORKLOADS:
   record=g.verify(30,True,external,workload)
   assert record["linked_kernel_sha256"]==clean["linked_kernel_sha256"]
   print("PASS external",workload,record["process_exit_status"],flush=True)
 finally:g.build_image=original
print("PASS one immutable artifact across the external workload corpus",flush=True)
