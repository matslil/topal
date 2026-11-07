"""QMP confirmation of controlled guest state after debugger disconnect."""
import json
import socket


def stopped_status(path):
    """Independently verify the controlled guest stays stopped after disconnect."""
    with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as client:
        client.settimeout(3)
        client.connect(str(path))
        with client.makefile('rb') as reader:
            if 'QMP' not in json.loads(reader.readline(65536)):
                raise RuntimeError('controlled guest lacks a QMP greeting')
            for command in ('qmp_capabilities','query-status'):
                client.sendall((json.dumps(dict(execute=command,id=command))+'\n').encode())
                for _ in range(32):
                    reply=json.loads(reader.readline(65536))
                    if reply.get('id')==command:break
                else:raise RuntimeError('QMP reply was not bounded')
                if 'error' in reply or 'return' not in reply:
                    raise RuntimeError('QMP command failed: '+repr(reply))
            status=reply['return']
            if status.get('running') is not False or status.get('status')!='debug':
                raise RuntimeError('controlled guest resumed after debugger observation: '+repr(status))
            return status
