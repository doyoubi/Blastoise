import socket
import struct
import cStringIO
import json


port = 8080
host = 'localhost'

s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.connect((host, port))

s.sendall("select * from book\n")

buff = cStringIO.StringIO()

while True:
    data = s.recv(1000)
    buff.write(data)
    if '\n' in data:
        break

output = buff.getvalue()
assert len(output) > 4
json_len = struct.unpack('<I', output[:4])[0]
print json_len
json_str = struct.unpack('%ds' % json_len, output[4:4+json_len])[0]
print json_str
tuple_desc = json.loads(json_str)
print tuple_desc

# payload
def attr_len(attr):
    if attr['type'] in ("Int", "Float"):
        return 4
    else:
        assert attr['type'] == 'Char'
        return int(attr['len'])

tuple_len = sum(map(attr_len, tuple_desc))
payload_start = 4 + json_len
tuple_sum = int(len(output) - payload_start) / int(tuple_len)
payload_end = tuple_len * tuple_sum + payload_start
assert len(output) == payload_end + 1
assert output[payload_end] == '\n'

def get_value(attr_type, data, index):
    if attr_type['type'] == 'Int':
        return struct.unpack('<I', data[index:index+4])[0]
    elif attr_type['type'] == 'Int':
        return struct.unpack('<f', data[index:index+4])[0]
    elif attr_type['type'] == 'Char':
        str_len = int(attr_type['len'])
        return struct.unpack('%ds' % str_len, data[index:index+str_len])[0].rstrip('\0')
    raise Exception('invalid type')

def get_gap(attr_type):
    if attr_type['type'] in ('Int', 'Float'):
        return 4
    elif attr_type['type'] == 'Char':
        return int(attr_type['len'])
    raise Exception('invalid type')

gaps = [0] + map(get_gap, tuple_desc)[:-1]
offset_list = map(lambda i: sum(gaps[:i]), range(1, len(gaps)+1))

for i in range(payload_start, payload_end, tuple_len):
    index_list = map(lambda o: o + i, offset_list)
    value_list = map(lambda (a, j): get_value(a, output, j), zip(tuple_desc, index_list))
    print value_list

