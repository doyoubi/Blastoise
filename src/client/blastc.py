import socket
import cStringIO
import sys
import cmd

from query import handle_query, print_tuple_desc, SqlError


class Console(cmd.Cmd):
    prompt = 'Blastoise > '

    def __init__(self, host, port):
        cmd.Cmd.__init__(self)  # Cmd is not new style
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.connect((host, port))
        self.tmp = ''

    def default(self, line):
        self.tmp += line
        if line[-1] != ';':
            return
        req = self.tmp[:-1] + '\n'
        print 'processing %s' % req
        self.tmp = ''
        self.socket.sendall(req)
        data = self.get_remote_data()
        try:
            (tuple_desc, tuple_list) = handle_query(data)
        except SqlError as e:
            print e.message
            return
        print_tuple_desc(tuple_desc)
        for t in tuple_list:
            print t

    def do_show(self, line):
        if line.strip() != 'tables':
            print 'only support show tables'
        else:
            self.socket.sendall('show tables\n')
            data = self.get_remote_data()
            print data

    def do_EOF(self, line):
        self.socket.close()
        return True

    def get_remote_data(self):
        buff = cStringIO.StringIO()
        while True:
            data = self.socket.recv(1000)
            buff.write(data)
            if '\r\n' in data:
                break
        data = buff.getvalue()
        buff.close()
        return data

def help():
    print 'usage: python blastc host port'
    sys.exit()


if __name__ == '__main__':
    if len(sys.argv) != 3:
        help()
    host = sys.argv[1]
    port = int(sys.argv[2])
    Console(host, port).cmdloop()

