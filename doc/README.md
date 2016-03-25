### packet format

#### query
(1) json_len (4 bytes Big-Endian)
(2) json_data (json_len bytes) this will indicate tuple_len
(3) tuple_data (tuple_len * n, n is the number of tuple) int and float is Big-Endian
(4) '\r\n'

json_len being zero means error occur, the format is
(1) 0 (also 4 bytes)
(2) error msg
(3) '\r\n'

#### non-query
(1) 0 (also 4 bytes)
(2) '\r\n'
