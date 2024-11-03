package cpu

import (
	"testing"

	"github.com/leakedmemory/mos6502/memory"
)

func TestLDAImmediatePositiveValue(t *testing.T) {
	acc := byte(0x42)
	offset := unreservedMemoryAddressStart
	mem := memory.Memory{}
	mem.Write(byte(ldaImmediateOpcode), offset)
	mem.Write(acc, offset+1)

	c := CPU{mem: &mem}
	c.Reset()

	pcInit := c.pc
	cyclesInit := c.cycles
	srInit := c.sr

	c.step()

	if c.acc != acc {
		t.Errorf("expected acc %q, actual acc %q", acc, c.acc)
	}

	bytesConsumed := c.pc - pcInit
	if bytesConsumed != ldaImmediateBytes {
		t.Errorf("bytes consumed: expected %d, actual %d", ldaImmediateBytes, bytesConsumed)
	}

	cyclesConsumed := c.cycles - cyclesInit
	if cyclesConsumed != ldaImmediateCycles {
		t.Errorf("cycles consumed: expected %d, actual %d", ldaImmediateCycles, cyclesConsumed)
	}

	if srInit != c.sr {
		t.Errorf("status register: expected %b, actual %b", srInit, c.sr)
	}
}
