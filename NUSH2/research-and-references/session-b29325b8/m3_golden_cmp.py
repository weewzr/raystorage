import sys, csv
a, b = sys.argv[1], sys.argv[2]
ra = open(a,'rb').read(); rb = open(b,'rb').read()
print('bytes', len(ra), len(rb), 'crlf', ra.count(b'\r\n'), rb.count(b'\r\n'))
print('identical after newline normalising:', ra.replace(b'\r\n', b'\n') == rb.replace(b'\r\n', b'\n'))
