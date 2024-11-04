package cpu

const (
	ldaImmediateBytes  uint16 = 2
	ldaImmediateCycles uint   = 2
)

func ldaSetStatusRegister(cpu *CPU) {
	cpu.sr &= ^(zeroSF | negativeSF)
	if cpu.acc == 0 {
		cpu.sr |= zeroSF
	} else if cpu.acc&0x80 != 0 {
		cpu.sr |= negativeSF
	}
}

// ldaImmediate loads a byte of memory into the accumulator.
//
// Attributes:
//
//	Bytes: 2
//	Cycles: 2
//	Flags affected: N, Z
func ldaImmediate(cpu *CPU) {
	cpu.acc = cpu.fetchByte()
	ldaSetStatusRegister(cpu)
}
