package cpu

import (
	"github.com/leakedmemory/mos6502/memory"
)

const unreservedMemoryAddressStart uint16 = 0x0200

const (
	defaultSP byte   = 0xFF
	defaultPC uint16 = unreservedMemoryAddressStart
	defaultSR byte   = 0x20
)

const (
	zeroSF     byte = 0x02
	negativeSF byte = 0x80
)

type (
	opcode      byte
	instruction func(*CPU)
)

type CPU struct {
	acc byte
	x   byte
	y   byte
	sp  byte
	pc  uint16
	// N, V, 1, B, D, I, Z, C
	sr     byte
	cycles uint
	mem    *memory.Memory
}

// Resets the CPU.
func (c *CPU) Reset() {
	c.acc = 0
	c.x = 0
	c.y = 0
	c.sp = defaultSP
	c.pc = defaultPC
	c.sr = defaultSR
	c.cycles = 7
}

// Runs the CPU.
func (c *CPU) Run() {
	for {
		c.step()
	}
}

func (c *CPU) step() {
	op := opcode(c.fetchByte())
	inst := c.decodeInstruction(op)
	inst(c)
}

func (c *CPU) fetchByte() byte {
	b := c.mem.Read(c.pc)
	c.cycles++
	c.pc++
	return b
}

func (c *CPU) decodeInstruction(op opcode) instruction {
	switch op {
	case ldaImmediateOpcode:
		return ldaImmediate
	default:
		panic("invalid opcode")
	}
}
