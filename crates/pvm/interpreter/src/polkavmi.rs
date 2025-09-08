use anyhow::Result;
use jam_codec::{Compact, Decode, Encode};
use polkavm::{
    Config, Engine, Gas as PolkavmGas, GasMeteringKind, InterruptKind, Module, ModuleConfig,
    ProgramBlob, ProgramCounter, ProgramParts, RawInstance, Reg,
};
use pvm::{
    host,
    score::{Gas, OpaqueHash},
    Argument, Invoked, Reason, State,
};
use std::borrow::Cow;

pub fn invoke<X: Argument>(
    mut ctx: X,
    code: &[u8],
    args: &[u8],
    gas: Gas,
    pc: usize,
) -> Result<Invoked<X>> {
    let jam_blob = PvmProgramBlob::from_bytes(code).unwrap();

    // Try to parse the extracted code as polkavm blob
    let mut parts = ProgramParts::default();
    parts.ro_data_size = jam_blob.ro_data.len() as u32;
    parts.rw_data_size = jam_blob.rw_data.len().next_multiple_of(4096) as u32
        + jam_blob.rw_data_padding_pages as u32 * 4096;
    parts.stack_size = jam_blob.stack_size;
    parts.ro_data = jam_blob.ro_data.into();
    parts.rw_data = jam_blob.rw_data.into();
    parts.code_and_jump_table = jam_blob.code_blob.into();
    parts.is_64_bit = true;

    let blob = ProgramBlob::from_parts(parts).unwrap();

    // Use the same workflow as polkavm examples
    let config = Config::from_env().unwrap_or_else(|_| Config::default());
    // config.set_allow_dynamic_paging(true);
    let engine = Engine::new(&config)?;
    let mut mconfig = ModuleConfig::default();
    mconfig.set_gas_metering(Some(GasMeteringKind::Sync));
    mconfig.set_step_tracing(true);
    // mconfig.set_dynamic_paging(true);
    let module = Module::from_blob(&engine, &mconfig, blob)?;

    // Use low-level API like the hello-world example
    let entry_point = ProgramCounter(pc as u32);
    let mut instance = module.instantiate()?;

    // Set up the execution environment
    instance.set_next_program_counter(entry_point);
    instance.set_gas(gas as PolkavmGas);

    instance.set_reg(Reg::RA, polkavm::RETURN_TO_HOST);
    let reg_sp = instance.module().memory_map().stack_address_high();
    instance.set_reg(Reg::SP, reg_sp as u64);

    // let reg_sl = instance.module().memory_map().stack_address_low();
    // let reg_hs = instance.module().memory_map().heap_base();
    // let ror = instance.module().memory_map().ro_data_range();
    // let rwr = instance.module().memory_map().rw_data_range();
    // let sr = instance.module().memory_map().stack_range();
    // let ar = instance.module().memory_map().aux_data_range();
    // let hb = instance.module().memory_map().heap_base();
    // let hs = instance.module().memory_map().max_heap_size();
    // s: MemoryInfo { read: 65536..78616, write: 196608..208896,
    // heap: 208896..4278050816, stack: 4278050816..4278059008, args: 4278124544..4278128640 }
    // p: Mmeory:        65536..81920  196608..208896
    // 208896-4294705152                4294828032..4294836224  4294901760..4294901760
    // println!("Mmeory: {:?} {:?} {:?} {:?} {}-{}", ror, rwr, sr, ar, hb, hb + hs);

    // Write args to memory
    let args_start = instance.module().memory_map().stack_address_low();
    instance.write_memory(args_start, args)?;
    instance.set_reg(Reg::A0, args_start as u64);
    instance.set_reg(Reg::A1, args.len() as u64);

    let mut output = Vec::new();
    let mut reason = Reason::Halt;
    let mut total_call_gas = 0;
    let mut count = 0;

    // registers: [4294901760, 4278059008, 0, 0, 0, 0, 0, 4278124544, 3, 0, 0, 0, 0]
    // registers: [4294901760, 4294836224, 0, 0, 0, 0, 0, 4294836220, 3, 0, 0, 0, 0]

    loop {
        let interrupt_kind = instance.run()?;

        match interrupt_kind {
            InterruptKind::Finished => {
                // Extract output from register A0 (common convention)
                let output_ptr = instance.reg(Reg::A0);
                let output_len = instance.reg(Reg::A1);

                output = if output_len > 0 && output_len <= 1024 * 1024 {
                    // Max 1MB output
                    let mut buf = vec![0u8; output_len as usize];
                    match instance.read_memory_into(output_ptr as u32, buf.as_mut_slice()) {
                        Ok(_) => buf,
                        Err(_) => Vec::new(),
                    }
                } else {
                    Vec::new()
                };

                break;
            }
            InterruptKind::NotEnoughGas => {
                println!("Unexpected not enough gas, count: {count}");
                reason = Reason::OOG;
                break;
            }
            InterruptKind::Segfault(_) => {
                println!("Unexpected segfault, count: {count}");
                reason = Reason::Fault { page: 0 };
                break;
            }
            InterruptKind::Trap => {
                println!("Program terminated with trap");
                reason = Reason::Halt;
                break;
            }
            InterruptKind::Ecalli(num) => {
                count -= 1;
                println!("step count: {count} call {num} pc {}", instance.program_counter().unwrap());
                // Handle host call directly with polkavm
                let call_gas = match num {
                    100 => 1,
                    _ => 11,
                };
                total_call_gas += call_gas;

                let call_result = polkavm_host_call(num, &mut instance, &mut ctx);

                if call_result != Reason::Continue {
                    // Extract output from register A0 (common convention)
                    let output_ptr = instance.reg(Reg::A0);
                    let output_len = instance.reg(Reg::A1);

                    output = if output_len > 0 && output_len <= 1024 * 1024 {
                        // Max 1MB output
                        let mut buf = vec![0u8; output_len as usize];
                        match instance.read_memory_into(output_ptr as u32, buf.as_mut_slice()) {
                            Ok(_) => buf,
                            Err(_) => Vec::new(),
                        }
                    } else {
                        Vec::new()
                    };

                    reason = call_result;
                    break;
                }
                continue;
            }
            InterruptKind::Step => {
                count += 1;
            }
        }
    }

    // Extract results
    let remaining_gas = instance.gas();
    let polkavm_gas_used = (gas as PolkavmGas) - remaining_gas;

    println!("polkavm gas: {polkavm_gas_used}, count: {count}, call gas: {total_call_gas}");

    let gas_used = count + total_call_gas;

    // Extract register state
    let mut registers = [0u64; 13];

    registers[0] = instance.reg(Reg::RA);
    registers[1] = instance.reg(Reg::SP);
    registers[2] = instance.reg(Reg::T0);
    registers[3] = instance.reg(Reg::T1);
    registers[4] = instance.reg(Reg::T2);
    registers[5] = instance.reg(Reg::S0);
    registers[6] = instance.reg(Reg::S1);
    registers[7] = instance.reg(Reg::A0);
    registers[8] = instance.reg(Reg::A1);
    registers[9] = instance.reg(Reg::A2);
    registers[10] = instance.reg(Reg::A3);
    registers[11] = instance.reg(Reg::A4);
    registers[12] = instance.reg(Reg::A5);

    let next_pc = instance
        .program_counter()
        .unwrap_or(ProgramCounter(pc as u32))
        .0 as usize;

    Ok(Invoked {
        output,
        reason,
        gas: gas_used as u64,
        data: ctx,
        state: State {
            pc: next_pc,
            gas: (gas as i64) - gas_used,
            registers,
            memory: parser::Memory::default(),
        },
    })
}

/// Handle host calls directly with polkavm instance
fn polkavm_host_call<X: Argument>(call: u32, instance: &mut RawInstance, ctx: &mut X) -> Reason {
    match call {
        0 => polkavm_host_gas(instance),
        1 => polkavm_host_fetch(instance, ctx),
        2 => polkavm_host_lookup(instance, ctx),
        3 => polkavm_host_read(instance, ctx),
        4 => polkavm_host_write(instance, ctx),
        5 => polkavm_host_info(instance, ctx),
        100 => polkavm_host_log(instance, ctx),
        _ => {
            tracing::warn!("unknown host call: {}", call);
            // Set register T1 (register[7] equivalent) to What error code
            instance.set_reg(Reg::T1, u64::MAX - 1); // What = u64::MAX - 1
            Reason::Continue
        }
    }
}

/// Host call 0: gas
fn polkavm_host_gas(instance: &mut RawInstance) -> Reason {
    let current_gas = instance.gas();
    instance.set_reg(Reg::A0, current_gas as u64);
    Reason::Continue
}

/// Host call 1: fetch
fn polkavm_host_fetch<X: Argument>(instance: &mut RawInstance, ctx: &mut X) -> Reason {
    // Create adapter but with corrected register mapping
    let mut adapter = PolkavmContextAdapter::new(instance, ctx);
    host::call(1, &mut adapter)
}

/// Host call 2: lookup
fn polkavm_host_lookup<X: Argument>(instance: &mut RawInstance, ctx: &mut X) -> Reason {
    let mut adapter = PolkavmContextAdapter::new(instance, ctx);
    host::call(2, &mut adapter)
}

/// Host call 3: read
fn polkavm_host_read<X: Argument>(instance: &mut RawInstance, ctx: &mut X) -> Reason {
    let mut adapter = PolkavmContextAdapter::new(instance, ctx);
    host::call(3, &mut adapter)
}

/// Host call 4: write
fn polkavm_host_write<X: Argument>(instance: &mut RawInstance, ctx: &mut X) -> Reason {
    let mut adapter = PolkavmContextAdapter::new(instance, ctx);
    host::call(4, &mut adapter)
}

/// Host call 5: info
fn polkavm_host_info<X: Argument>(instance: &mut RawInstance, ctx: &mut X) -> Reason {
    let mut adapter = PolkavmContextAdapter::new(instance, ctx);
    host::call(5, &mut adapter)
}

/// Host call 100: log
fn polkavm_host_log<X: Argument>(instance: &mut RawInstance, ctx: &mut X) -> Reason {
    let mut adapter = PolkavmContextAdapter::new(instance, ctx);
    // host::call(100, &mut adapter);
    adapter.rset(7, 0);
    Reason::Continue
    // let ptr = instance.reg(Reg::A0) as u32;
    // let len = instance.reg(Reg::A1) as u32;

    // if len > 0 && len <= 1024 { // Max 1KB log message
    //     match instance.read_memory(ptr, len) {
    //         Ok(data) => {
    //             match std::str::from_utf8(&data) {
    //                 Ok(msg) => {
    //                     tracing::info!("PVM LOG: {}", msg);
    //                     instance.set_reg(Reg::T1, 0); // Ok = 0
    //                 }
    //                 Err(_) => {
    //                     tracing::warn!("PVM LOG: invalid UTF-8");
    //                     instance.set_reg(Reg::T1, u64::MAX - 1); // What
    //                 }
    //             }
    //         }
    //         Err(_) => {
    //             instance.set_reg(Reg::T1, u64::MAX - 2); // OOB (out of bounds)
    //         }
    //     }
    // } else {
    //     instance.set_reg(Reg::T1, u64::MAX - 1); // What (invalid length)
    // }

    // Reason::Continue
}

/// Adapter that bridges polkavm instance with space-vm Argument interface
struct PolkavmContextAdapter<'a, X: Argument> {
    instance: &'a mut RawInstance,
    ctx: &'a mut X,
}

impl<'a, X: Argument> PolkavmContextAdapter<'a, X> {
    fn new(instance: &'a mut RawInstance, ctx: &'a mut X) -> Self {
        Self { instance, ctx }
    }
}

impl<'a, X: Argument> Argument for PolkavmContextAdapter<'a, X> {
    const SUPPORTED_CALLS: &'static [u32] = X::SUPPORTED_CALLS;
    const INITIAL_PC: u64 = X::INITIAL_PC;

    fn burn(&mut self, _gas: pvm::score::Gas) {
        // Only burn gas in the external context, let polkavm manage its own gas
        // This avoids double-burning gas which could cause different consumption patterns
        panic!("NO burn");
        // self.ctx.burn(gas);
    }

    fn read(&self, address: u32, len: u32) -> Result<Vec<u8>> {
        // Read from polkavm memory
        Ok(self.instance.read_memory(address, len).unwrap())
    }

    fn write(&mut self, address: u32, data: &[u8]) -> Result<()> {
        // Write to polkavm memory
        Ok(self.instance.write_memory(address, data).unwrap())
    }

    fn rget(&self, reg: u8) -> u64 {
        // Map space-vm register indices to polkavm Reg enum indices exactly
        // polkavm Reg enum: RA=0, SP=1, T0=2, T1=3, T2=4, S0=5, S1=6, A0=7, A1=8, A2=9, A3=10, A4=11, A5=12
        match reg {
            0 => self.instance.reg(Reg::RA),  // RA = 0
            1 => self.instance.reg(Reg::SP),  // SP = 1
            2 => self.instance.reg(Reg::T0),  // T0 = 2
            3 => self.instance.reg(Reg::T1),  // T1 = 3
            4 => self.instance.reg(Reg::T2),  // T2 = 4
            5 => self.instance.reg(Reg::S0),  // S0 = 5
            6 => self.instance.reg(Reg::S1),  // S1 = 6
            7 => self.instance.reg(Reg::A0),  // A0 = 7
            8 => self.instance.reg(Reg::A1),  // A1 = 8
            9 => self.instance.reg(Reg::A2),  // A2 = 9
            10 => self.instance.reg(Reg::A3), // A3 = 9
            11 => self.instance.reg(Reg::A4), // A4 = 11
            12 => self.instance.reg(Reg::A5), // A5 = 12
            _ => 0,                           // Invalid register
        }
    }

    fn rset(&mut self, reg: u8, value: u64) {
        // Map space-vm register indices to polkavm Reg enum indices exactly
        // polkavm Reg enum: RA=0, SP=1, T0=2, T1=3, T2=4, S0=5, S1=6, A0=7, A1=8, A2=9, A3=10, A4=11, A5=12
        match reg {
            0 => self.instance.set_reg(Reg::RA, value),  // RA = 0
            1 => self.instance.set_reg(Reg::SP, value),  // SP = 1
            2 => self.instance.set_reg(Reg::T0, value),  // T0 = 2
            3 => self.instance.set_reg(Reg::T1, value),  // T1 = 3
            4 => self.instance.set_reg(Reg::T2, value),  // T2 = 4
            5 => self.instance.set_reg(Reg::S0, value),  // S0 = 5
            6 => self.instance.set_reg(Reg::S1, value),  // S1 = 6
            7 => self.instance.set_reg(Reg::A0, value),  // A0 = 7
            8 => self.instance.set_reg(Reg::A1, value),  // A1 = 8
            9 => self.instance.set_reg(Reg::A2, value),  // A2 = 9
            10 => self.instance.set_reg(Reg::A3, value), // A3 = 10
            11 => self.instance.set_reg(Reg::A4, value), // A4 = 11
            12 => self.instance.set_reg(Reg::A5, value), // A5 = 12
            _ => {}                                      // Invalid register
        }
    }

    fn heap_ptr(&self) -> u32 {
        // Use polkavm heap base
        self.instance.module().memory_map().heap_base()
    }

    fn set_heap_ptr(&mut self, heap_ptr: u32) {
        // Polkavm manages heap internally, but we can try to allocate to the target
        let current_heap = self.instance.module().memory_map().heap_base();
        if heap_ptr > current_heap {
            let _ = self.instance.sbrk(heap_ptr - current_heap);
        }
    }

    fn allocate(&mut self, start: u32, count: u32) -> Result<()> {
        // For polkavm, we don't need explicit allocation as memory is managed automatically
        // We could validate that the range is accessible
        let end = start.saturating_add(count);
        if self.instance.is_memory_accessible(start, count, true) {
            Ok(())
        } else {
            // Try to expand heap if needed
            let heap_base = self.instance.module().memory_map().heap_base();
            if start >= heap_base {
                let needed = end.saturating_sub(heap_base + self.instance.heap_size());
                if needed > 0 {
                    self.instance
                        .sbrk(needed)
                        .map_err(|e| anyhow::anyhow!("Allocation failed: {:?}", e))?;
                }
            }
            Ok(())
        }
    }

    fn account(&mut self, id: u64) -> Result<&mut impl pvm::score::Account> {
        self.ctx.account(id)
    }

    fn this(&mut self) -> Result<&mut impl pvm::score::Account> {
        self.ctx.this()
    }

    fn entropy(&self) -> OpaqueHash {
        self.ctx.entropy()
    }

    fn operands(&self) -> &[score::vm::Operand] {
        self.ctx.operands()
    }

    fn service(&self) -> score::ServiceId {
        self.ctx.service()
    }
}

/// Read `ConventionalMetadata` as bytes.
pub fn read_metadata_slice<'a>(bytes: &mut &'a [u8]) -> Option<&'a [u8]> {
    let offset = read_var(bytes)?;
    let metadata = read_slice(bytes, offset)?;
    Some(metadata)
}

/// A JAM-specific program blob.
pub struct PvmProgramBlob<'a> {
    pub metadata: Cow<'a, [u8]>,
    pub ro_data: Cow<'a, [u8]>,
    pub rw_data: Cow<'a, [u8]>,
    pub code_blob: Cow<'a, [u8]>,
    pub rw_data_padding_pages: u16,
    pub stack_size: u32,
}

fn read_u24(bytes: &mut &[u8]) -> Option<u32> {
    let xs = bytes.get(..3)?;
    *bytes = &bytes[3..];
    Some(u32::from_le_bytes([xs[0], xs[1], xs[2], 0]))
}

fn write_u24(value: u32, output: &mut Vec<u8>) -> Result<(), ()> {
    if value >= (1 << 24) {
        return Err(());
    }

    output.extend_from_slice(&value.to_le_bytes()[0..3]);
    Ok(())
}

fn read_u16(bytes: &mut &[u8]) -> Option<u16> {
    let xs = bytes.get(..2)?;
    *bytes = &bytes[2..];
    Some(u16::from_le_bytes([xs[0], xs[1]]))
}

fn read_u32(bytes: &mut &[u8]) -> Option<u32> {
    let xs = bytes.get(..4)?;
    *bytes = &bytes[4..];
    Some(u32::from_le_bytes([xs[0], xs[1], xs[2], xs[3]]))
}

fn read_var(bytes: &mut &[u8]) -> Option<u32> {
    Some(Compact::<u32>::decode(bytes).ok()?.0)
}

fn write_var(value: u32, output: &mut Vec<u8>) {
    Compact::<u32>(value).encode_to(output)
}

fn read_cow<'a>(bytes: &mut &'a [u8], length: u32) -> Option<Cow<'a, [u8]>> {
    read_slice(bytes, length).map(Cow::Borrowed)
}

fn read_slice<'a>(bytes: &mut &'a [u8], length: u32) -> Option<&'a [u8]> {
    let length = length as usize;
    let slice = bytes.get(..length)?;
    *bytes = &bytes[length..];
    Some(slice)
}

impl<'a> PvmProgramBlob<'a> {
    pub fn from_bytes(mut bytes: &'a [u8]) -> Option<Self> {
        let metadata = Cow::Borrowed(read_metadata_slice(&mut bytes)?);
        let ro_data_len = read_u24(&mut bytes)?;
        let rw_data_len = read_u24(&mut bytes)?;
        let rw_data_padding_pages = read_u16(&mut bytes)?;
        let stack_size = read_u24(&mut bytes)?;
        let ro_data = read_cow(&mut bytes, ro_data_len)?;
        let rw_data = read_cow(&mut bytes, rw_data_len)?;
        let code_blob_len = read_u32(&mut bytes)?;
        let code_blob = read_cow(&mut bytes, code_blob_len)?;

        if !bytes.is_empty() {
            return None;
        }

        Some(PvmProgramBlob {
            metadata,
            rw_data_padding_pages,
            stack_size,
            ro_data,
            rw_data,
            code_blob,
        })
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, &'static str> {
        let mut output = Vec::new();
        write_var(
            u32::try_from(self.metadata.len()).map_err(|_| "metadata too large")?,
            &mut output,
        );
        output.extend_from_slice(&self.metadata);
        write_u24(
            u32::try_from(self.ro_data.len()).map_err(|_| "too large RO data")?,
            &mut output,
        )
        .map_err(|_| "too large RO data")?;
        write_u24(
            u32::try_from(self.rw_data.len()).map_err(|_| "too large RW data")?,
            &mut output,
        )
        .map_err(|_| "too large RW data")?;
        output.extend_from_slice(&self.rw_data_padding_pages.to_le_bytes());
        write_u24(self.stack_size, &mut output).map_err(|_| "too large stack size")?;
        output.extend_from_slice(&self.ro_data);
        output.extend_from_slice(&self.rw_data);
        output.extend_from_slice(
            &u32::try_from(self.code_blob.len())
                .map_err(|_| "too large code")?
                .to_le_bytes(),
        );
        output.extend_from_slice(&self.code_blob);
        Ok(output)
    }
}

impl From<PvmProgramBlob<'_>> for polkavm::ProgramParts {
    fn from(other: PvmProgramBlob<'_>) -> Self {
        let mut parts = polkavm::ProgramParts::default();
        parts.ro_data_size = other.ro_data.len() as u32;
        parts.rw_data_size = other.rw_data.len().next_multiple_of(4096) as u32
            + other.rw_data_padding_pages as u32 * 4096;
        parts.stack_size = other.stack_size;
        parts.ro_data = other.ro_data.into();
        parts.rw_data = other.rw_data.into();
        parts.code_and_jump_table = other.code_blob.into();
        parts.is_64_bit = true;
        parts
    }
}

impl<'a> PvmProgramBlob<'a> {
    pub fn from_pvm(parts: &'a polkavm::ProgramParts, metadata: Cow<'a, [u8]>) -> Self {
        // Pad RO section with zeroes.
        let mut ro_data = parts.ro_data.to_vec();
        ro_data.resize(parts.ro_data_size as usize, 0);
        // Calculate the padding for RW section.
        let padding = (parts.rw_data_size as usize).next_multiple_of(4096)
            - parts.rw_data.len().next_multiple_of(4096);
        let rw_data_padding_pages = padding / 4096;
        let rw_data_padding_pages = rw_data_padding_pages
            .try_into()
            .expect("The RW data section is too big");
        Self {
            metadata,
            ro_data: ro_data.into(),
            rw_data: (&parts.rw_data[..]).into(),
            code_blob: (&parts.code_and_jump_table[..]).into(),
            rw_data_padding_pages,
            stack_size: parts.stack_size,
        }
    }
}
