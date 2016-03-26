import struct
import json


class SqlError(Exception):
    pass


def handle_query(response):
    assert len(response) > 4
    json_len = struct.unpack('<I', response[:4])[0]
    if json_len == 0:
        raise SqlError(response[4:])
    json_str = struct.unpack('%ds' % json_len, response[4:4+json_len])[0]
    tuple_desc = json.loads(json_str)
    tuple_len = sum(map(attr_len, tuple_desc))
    payload_start = 4 + json_len
    tuple_sum = int(len(response) - payload_start) / int(tuple_len)
    payload_end = tuple_len * tuple_sum + payload_start
    assert len(response) == payload_end + 2
    assert response[payload_end:] == '\r\n'
    
    gaps = [0] + map(get_gap, tuple_desc)[:-1]
    offset_list = map(lambda i: sum(gaps[:i]), range(1, len(gaps)+1))
    
    # payload
    result = []
    for i in range(payload_start, payload_end, tuple_len):
        index_list = map(lambda o: o + i, offset_list)
        value_list = map(lambda (a, j): get_value(a, response, j), zip(tuple_desc, index_list))
        result.append(tuple(value_list))

    return (tuple_desc, result)


def attr_len(attr):
    if attr['type'] in ("Int", "Float"):
        return 4
    else:
        assert attr['type'] == 'Char'
        return int(attr['len'])


def get_value(attr_type, data, index):
    if attr_type['type'] == 'Int':
        return struct.unpack('<I', data[index:index+4])[0]
    elif attr_type['type'] == 'Float':
        return struct.unpack('<f', data[index:index+4])[0]
    elif attr_type['type'] == 'Char':
        str_len = int(attr_type['len'])
        return struct.unpack('%ds' % str_len, data[index:index+str_len])[0].rstrip('\0')
    raise Exception('invalid type %s' % attr_type['type'])


def get_gap(attr_type):
    if attr_type['type'] in ('Int', 'Float'):
        return 4
    elif attr_type['type'] == 'Char':
        return int(attr_type['len'])
    raise Exception('invalid type %s' % attr_type['type'])


def print_tuple_desc(tuple_desc):
    print ', '.join(map(repr_attr_type, tuple_desc))


def repr_attr_type(attr_type):
    if attr_type['type'] in ('Int', 'Float'):
        return attr_type['type']
    elif attr_type['type'] == 'Char':
        return 'Char(%s)' % attr_type['len']
    raise Exception('invalid type')
