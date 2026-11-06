import importlib.util
spec=importlib.util.spec_from_file_location('gate','linux-kernel/labs/qemu/x86_64/topal_gate.py')
g=importlib.util.module_from_spec(spec)
spec.loader.exec_module(g)
for mode in ('empty','null'):
    result=g.verify(30,True,None,'true',mode)
    print('PASS clean',mode,result['linked_kernel_sha256'],flush=True)
