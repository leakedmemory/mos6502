package cpu

const (
	ldaImmediateBytes  uint16 = 2
	ldaImmediateCycles uint64 = 2
)

func ldaSetStatusRegister(cpu *CPU) {
	cpu.sr &= ^(zeroSF | negativeSF)
	if cpu.acc == 0 {
		cpu.sr |= zeroSF
	} else if cpu.acc&0x80 != 0 {
		cpu.sr |= negativeSF
	}
}

// ldaImmediate consumes 2 bytes and 2 cycles.
func ldaImmediate(cpu *CPU) {
	cpu.acc = cpu.fetchByte()
	ldaSetStatusRegister(cpu)
}
