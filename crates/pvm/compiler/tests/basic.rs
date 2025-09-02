use pvm::Memory;
use pvmc::Compiler;

/// Test program from jam-test-vectors/pvm/programs/inst_load_imm.json
const LOAD_IMM_PROGRAM: &[u8] = &[0, 0, 10, 20, 7, 239, 190, 173, 222, 0, 0, 0, 0, 1, 0];

/// Test program from jam-test-vectors/pvm/programs/inst_add_imm_32.json
const ADD_IMM_32_PROGRAM: &[u8] = &[0, 0, 3, 131, 121, 2, 1];

#[test]
fn test_load_imm() -> anyhow::Result<()> {
    let module = Compiler.compile(&pvm::Program {
        code: LOAD_IMM_PROGRAM.to_vec(),
        registers: [0; pvm::REGISTER_COUNT],
        memory: Memory::default(),
    })?;
    let result = module.invoke(&[0; pvm::REGISTER_COUNT], 0, 10000, Memory::default())?;

    // Expected registers from test vector
    let expected = [0, 0, 0, 0, 0, 0, 0, 3735928559, 0, 0, 0, 0, 0];
    assert_eq!(result.registers, expected);
    Ok(())
}

#[test]
fn test_add_imm_32() -> anyhow::Result<()> {
    let module = Compiler.compile(&pvm::Program {
        code: ADD_IMM_32_PROGRAM.to_vec(),
        registers: [0; pvm::REGISTER_COUNT],
        memory: Memory::default(),
    })?;
    let result = module.invoke(&[0; pvm::REGISTER_COUNT], 0, 10000, Memory::default())?;

    // With zero initialization, register 9 should contain 0 + 2 = 2 (from add_imm_32 instruction)
    let expected = [0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0];
    assert_eq!(result.registers, expected);
    Ok(())
}
