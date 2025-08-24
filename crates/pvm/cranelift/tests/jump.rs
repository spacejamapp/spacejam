//! Test the table construction

use anyhow::Result;
use cranelift::prelude::{types::I32, *};
use cranelift_codegen::{ir::BlockCall, Context};

/// Allocate executable memory
fn alloc_exec(size: usize) -> Result<*mut u8> {
    unsafe {
        let ptr = libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        );

        if ptr == libc::MAP_FAILED {
            anyhow::bail!("Failed to allocate executable memory");
        }

        Ok(ptr as *mut u8)
    }
}

fn create_isa() -> Result<isa::OwnedTargetIsa> {
    let mut flag_builder = settings::builder();
    flag_builder
        .set("use_colocated_libcalls", "false")
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    flag_builder
        .set("is_pic", "false")
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let isa_builder = cranelift_native::builder().map_err(|e| anyhow::anyhow!("{}", e))?;
    let isa = isa_builder.finish(settings::Flags::new(flag_builder))?;
    Ok(isa)
}

fn exec(codegen: fn(&mut FunctionBuilder), args: *const u32) -> Result<i32> {
    let isa = create_isa()?;
    let mut ctx = Context::new();
    ctx.func.signature.params = vec![AbiParam::new(I32)];
    ctx.func.signature.returns = vec![AbiParam::new(I32)];
    let mut bctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut bctx);

    // compile the function
    codegen(&mut builder);
    builder.finalize();
    let mut ctrl = cranelift_codegen::control::ControlPlane::default();
    ctx.compile(&*isa, &mut ctrl).expect("failed to compile");

    // execute the code
    let code = ctx.compiled_code().expect("failed to get compiled code");
    let buffer = code.buffer.data();
    let ptr = alloc_exec(buffer.len())?;
    unsafe {
        ptr.copy_from_nonoverlapping(buffer.as_ptr(), buffer.len());
    }
    let func = unsafe { std::mem::transmute::<*mut u8, fn(*const u32) -> i32>(ptr) };
    Ok(func(args))
}

#[test]
fn test_ret() {
    fn program(builder: &mut FunctionBuilder) {
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);

        // just return input value
        let value = builder.block_params(entry)[0];
        builder.switch_to_block(entry);
        builder.ins().return_(&[value]);
        builder.seal_block(entry);
    }

    let data = 42;
    let result = exec(program, data as *const u32).expect("failed to execute program");
    assert_eq!(result, 42);
}

#[test]
fn test_jump_table() {
    fn program(builder: &mut FunctionBuilder) {
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);

        // Create 4 blocks that return different constants
        let block0 = builder.create_block();
        let block1 = builder.create_block();
        let block2 = builder.create_block();
        let default_block = builder.create_block();

        // Create BlockCall objects for jump table
        let bc_block0 = BlockCall::new(
            block0,
            std::iter::empty(),
            &mut builder.func.dfg.value_lists,
        );
        let bc_block1 = BlockCall::new(
            block1,
            std::iter::empty(),
            &mut builder.func.dfg.value_lists,
        );
        let bc_block2 = BlockCall::new(
            block2,
            std::iter::empty(),
            &mut builder.func.dfg.value_lists,
        );
        let bc_default = BlockCall::new(
            default_block,
            std::iter::empty(),
            &mut builder.func.dfg.value_lists,
        );

        // Create jump table: default=default_block, entries=[block0, block1, block2]
        // - Index 0 -> block0
        // - Index 1 -> block1
        // - Index 2 -> block2
        // - Index >= 3 -> default_block
        let jt_data = JumpTableData::new(bc_default, &[bc_block0, bc_block1, bc_block2]);
        let jt = builder.create_jump_table(jt_data);

        // br_table: jump to blocks based on input value directly
        builder.switch_to_block(entry);
        let value = builder.block_params(entry)[0];
        builder.ins().br_table(value, jt);
        {
            // Block0: return 10
            builder.switch_to_block(block0);
            let ten = builder.ins().iconst(I32, 10);
            builder.ins().return_(&[ten]);
            builder.seal_block(block0);

            // Block1: return 20
            builder.switch_to_block(block1);
            let twenty = builder.ins().iconst(I32, 20);
            builder.ins().return_(&[twenty]);
            builder.seal_block(block1);

            // Block2: return 30
            builder.switch_to_block(block2);
            let thirty = builder.ins().iconst(I32, 30);
            builder.ins().return_(&[thirty]);
            builder.seal_block(block2);

            // Default block: return 40
            builder.switch_to_block(default_block);
            let forty = builder.ins().iconst(I32, 40);
            builder.ins().return_(&[forty]);
            builder.seal_block(default_block);
        }

        builder.seal_block(entry);
    }

    // Test index 0 -> should return 10
    let data = 0;
    let result = exec(program, data as *const u32).expect("failed to execute program");
    assert_eq!(result, 10);

    // Test index 1 -> should return 20
    let data = 1;
    let result = exec(program, data as *const u32).expect("failed to execute program");
    assert_eq!(result, 20);

    // Test index 2 -> should return 30
    let data = 2;
    let result = exec(program, data as *const u32).expect("failed to execute program");
    assert_eq!(result, 30);

    // Test index >= 3 -> should return 40 (default)
    let data = 3;
    let result = exec(program, data as *const u32).expect("failed to execute program");
    assert_eq!(result, 40);

    let data = 100;
    let result = exec(program, data as *const u32).expect("failed to execute program");
    assert_eq!(result, 40);
}
