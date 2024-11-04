package cpu

import (
	"testing"

	"github.com/leakedmemory/mos6502/memory"
)

type ldaImmediateTest struct {
	acc    byte
	sr     byte
	bytes  uint16
	cycles uint
}

func newLDAImmediateTest(acc byte) *ldaImmediateTest {
	return &ldaImmediateTest{
		acc:    acc,
		sr:     defaultSR,
		bytes:  ldaImmediateBytes,
		cycles: ldaImmediateCycles,
	}
}

func ldaImmediateTestHelper(acc byte) *ldaImmediateTest {
	offset := unreservedMemoryAddressStart
	mem := memory.Memory{}
	mem.Write(byte(ldaImmediateOpcode), offset)
	mem.Write(acc, offset+1)

	c := CPU{mem: &mem}
	c.Reset()

	pcInit := c.pc
	cyclesInit := c.cycles

	c.step()

	return &ldaImmediateTest{
		acc:    c.acc,
		sr:     c.sr,
		bytes:  c.pc - pcInit,
		cycles: c.cycles - cyclesInit,
	}
}

func TestLDAImmediateWithPositiveValue(t *testing.T) {
	var acc byte = 0x42
	expected := newLDAImmediateTest(acc)
	actual := ldaImmediateTestHelper(acc)

	if *expected != *actual {
		t.Errorf("expected %+v, actual %+v\n", expected, actual)
	}
}

func TestLDAImmediateWithNegativeValue(t *testing.T) {
	var acc byte = 0x82
	expected := newLDAImmediateTest(acc)
	expected.sr |= negativeSF
	actual := ldaImmediateTestHelper(acc)

	if *expected != *actual {
		t.Errorf("expected %+v, actual %+v\n", expected, actual)
	}
}

func TestLDAImmediateWithZero(t *testing.T) {
	var acc byte = 0x00
	expected := newLDAImmediateTest(acc)
	expected.sr |= zeroSF
	actual := ldaImmediateTestHelper(acc)

	if *expected != *actual {
		t.Errorf("expected %+v, actual %+v\n", expected, actual)
	}
}
