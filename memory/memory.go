package memory

import "os"

// 64 KiB.
const memorySize uint = 64 * 1024

// TODO: maybe split into RAM and ROM and create a bus?
//
//nolint:godox
type Memory [memorySize]byte

// Write changes the content of addr in memory to val.
func (m *Memory) Write(val byte, addr uint16) {
	m[addr] = val
}

// Read returns the content from addr in memory.
func (m *Memory) Read(addr uint16) byte {
	return m[addr]
}

// Dump writes the memory content to the specified file.
//
//nolint:godox
func (m *Memory) Dump(f *os.File) error {
	// TODO
	panic("unimplemented")
}
