#!/usr/bin/env python3

import os
import sys

filename = sys.argv[1]
orig_size = os.path.getsize(filename)

with open("src/settings.json") as fd:
	data = fd.read().encode('utf-8')

with open(filename, mode='r+b') as fd:
	fd.seek(-12, os.SEEK_END)
	magic = int.from_bytes(fd.read(4), byteorder='big')
	if magic != 0xDEADBEEF:
		print("No magic found, appending")
		fd.seek(0, os.SEEK_END)
	else:
		print("Magic found, overwriting")
		orig_size = int.from_bytes(fd.read(8), byteorder='big')
		fd.seek(orig_size)

	print(f"Original size {orig_size}, writing {len(data)} bytes + footer (12 bytes)")

	fd.write(data)
	fd.write((0xDEADBEEF).to_bytes(4, byteorder='big'))
	fd.write(orig_size.to_bytes(8, byteorder='big'))

print("Done")
