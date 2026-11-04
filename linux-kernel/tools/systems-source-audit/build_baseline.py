"""Build independent native artifact evidence with the qualified predecessor."""

from pathlib import Path
import subprocess,sys,json
repo=Path(__file__).resolve().parents[3];lab=repo/'linux-kernel/labs/qemu/x86_64'
sys.path.insert(0,str(lab));import topal_gate
import argparse
parser=argparse.ArgumentParser(description="Build the pinned predecessor against the unchanged TK277 Topal sources")
parser.add_argument("--checkout",type=Path,required=True)
parser.add_argument("--output",type=Path,required=True)
args=parser.parse_args()
root=args.output.resolve();root.mkdir(exist_ok=False)
checkout=args.checkout.resolve()
revision=subprocess.run(["git","-C",str(checkout),"rev-parse","HEAD"],check=True,capture_output=True,text=True).stdout.strip()
assert revision=="73e17ab5a50d34561b71db6fb835ed0aae31217f"
subprocess.run(["git","diff","--exit-code",revision,"--","linux-kernel/kernel"],cwd=repo,check=True)
command=['cargo','run','--quiet','--manifest-path',str(checkout/'Cargo.toml'),
         '-p','topal-compiler','--bin','topalc','--','--target','x86_64-unknown-none',
         '--cpu','generic','--board','topal-qemu-pc-q35-10.2','--emit','executable',
         '--library-root',str(repo/'linux-kernel/kernel')]
subprocess.run(command+['-o',str(root/'generated'),str(topal_gate.SOURCE_PATH)],check=True)
initial=bytearray((root/'generated/initial-user.elf').read_bytes());initial[4097]=0xa5
(root/'initial.elf').write_bytes(initial)
(root/'static.elf').write_bytes(topal_gate.static_user_fixture())
subprocess.run(command+['--initial-user-image',str(root/'initial.elf'),
                      '--static-user-image',str(root/'static.elf'),'-o',str(root/'linked'),
                      str(topal_gate.SOURCE_PATH)],check=True)
