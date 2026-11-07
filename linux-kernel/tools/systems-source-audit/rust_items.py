from pathlib import Path
import re

def masked_rust(text, literals=None):
    masked = list(text)
    pos = 0
    while pos < len(text):
        end = None
        if text.startswith('//', pos):
            end = text.find('\n', pos)
            if end < 0: end = len(text)
        elif text.startswith('/*', pos):
            end, nesting = pos + 2, 1
            while end < len(text) and nesting:
                if text.startswith('/*', end): nesting += 1; end += 2
                elif text.startswith('*/', end): nesting -= 1; end += 2
                else: end += 1
            assert nesting == 0
        else:
            raw = re.match(r'(?:br|r)(#{0,255})"', text[pos:pos+260])
            if raw:
                delimiter = '"' + raw[1]
                at = text.find(delimiter, pos + raw.end())
                assert at >= 0
                end = at + len(delimiter)
                if literals is not None: literals.append((pos, end, text[pos:end]))
            elif text[pos] == '"':
                end = pos + 1
                while end < len(text):
                    if text[end] == '\\': end += 2
                    elif text[end] == '"': end += 1; break
                    else: end += 1
                if literals is not None: literals.append((pos, end, text[pos:end]))
            elif text[pos] == "'":
                literal = re.match(r"'(?:[^'\\\n]|\\(?:[nrt0\\\"']|x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}))'", text[pos:pos+32])
                if literal:
                    end = pos + literal.end()
                    if literals is not None: literals.append((pos, end, text[pos:end]))
        if end is None: pos += 1
        else:
            for i in range(pos, end):
                if masked[i] != '\n': masked[i] = ' '
            pos = end
    return ''.join(masked)

def items(text):
    mask = masked_rust(text)
    start = pos = 0
    result = []
    while pos < len(mask):
        while pos < len(mask) and mask[pos].isspace(): pos += 1
        if pos == len(mask):
            if result: result[-1] += text[start:]
            else: assert not text.strip()
            break
        stack = []
        while pos < len(mask):
            char = mask[pos]
            if char in '({[': stack.append(char)
            elif char in ')}]':
                assert stack and '({['[')}]'.index(char)] == stack[-1], (pos, char)
                stack.pop()
                if char == '}' and not stack:
                    pos += 1
                    tail = pos
                    while tail < len(mask) and mask[tail].isspace(): tail += 1
                    if tail < len(mask) and mask[tail] == ';': pos = tail + 1
                    break
            elif char == ';' and not stack:
                pos += 1
                break
            pos += 1
        else: raise AssertionError('unterminated item '+text[start:start+100])
        result.append(text[start:pos])
        start = pos
    assert ''.join(result) == text
    return result

def body(item):
    mask = masked_rust(item)
    at = mask.index('{')
    end = mask.rindex('}')
    return item[:at+1], item[at+1:end], item[end:]

def descriptor(item):
    mask = masked_rust(item)
    match = re.search(r'\b(?:fn|struct|enum|type|mod|impl|const)\s+([A-Za-z_][A-Za-z_0-9]*)', mask)
    return match[1] if match else mask.strip().splitlines()[0][:80]
