//! Build script for the RISC-V parser

use anyhow::Result;
use heck::ToUpperCamelCase;
use proc_macro2::Span;
use quote::ToTokens;
use std::{env, fs, path::PathBuf, process::Command};
use syn::{parse_quote, Expr, Ident, ItemEnum};

const RISCV_OPCODES_REPO: &str = "https://github.com/riscv/riscv-opcodes.git";
const PARSE_ARGS: [&str; 3] = ["-rust", "rv_i", "rv_m"];

fn main() -> Result<()> {
    let mut ctx = BuildContext::new()?;
    ctx.download_opcodes()?;
    ctx.read_instructions()?;
    ctx.write_instr_rs()?;
    Ok(())
}

/// Opcodes build context
#[derive(Default)]
struct BuildContext {
    root: PathBuf,
    instr_rs: PathBuf,
    r: Vec<Instr>,
    i: Vec<Instr>,
    s: Vec<Instr>,
    b: Vec<Instr>,
    u: Vec<Instr>,
    j: Vec<Instr>,
}

impl BuildContext {
    fn new() -> Result<Self> {
        let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
        let out_dir = PathBuf::from(env::var("OUT_DIR")?);
        Ok(Self {
            root,
            instr_rs: out_dir.join("instr.rs"),
            ..Default::default()
        })
    }

    fn write_instr_rs(&self) -> Result<()> {
        let mut instr_rs = String::new();
        instr_rs.push_str("use crate::format::{RType, IType, SType, BType, UType, JType};\n");

        let mut item_enum: ItemEnum = parse_quote!(
            /// RISC-V instruction
            #[derive(Debug, PartialEq, Eq, Clone, Copy)]
            pub enum Instruction {}
        );

        for (fmt, instructions) in [
            ("R", &self.r),
            ("I", &self.i),
            ("S", &self.s),
            ("B", &self.b),
            ("U", &self.u),
            ("J", &self.j),
        ] {
            for instr in instructions {
                let format = Ident::new(&format!("{fmt}Type"), Span::call_site());
                let name = Ident::new(&instr.name.to_upper_camel_case(), Span::call_site());
                let variant = parse_quote!(#name(#format));
                item_enum.variants.push(variant);

                // TODO: update parse
            }
        }

        instr_rs.push_str(item_enum.to_token_stream().to_string().as_str());

        fs::write(&self.instr_rs, instr_rs)?;
        Ok(())
    }

    /// Read the instructions from the `inst.rs` file
    fn read_instructions(&mut self) -> Result<()> {
        let inst_rs = self.root.join("riscv-opcodes/inst.rs");
        let contents = std::fs::read_to_string(inst_rs)?.clone();
        let mut lines = contents.lines().skip(2);
        while let (Some(march), Some(mask)) = (lines.next(), lines.next()) {
            if !march.starts_with("const MATCH_") || !mask.starts_with("const MASK_") {
                break;
            }

            let (name, march) = {
                let mut matches = march.split_ascii_whitespace();
                let name = matches
                    .nth(1)
                    .expect("Failed to parse name")
                    .trim_start_matches("MATCH_")
                    .trim();

                let march = matches
                    .last()
                    .expect("Failed to parse match")
                    .trim_end_matches(';')
                    .trim();

                (name, march)
            };

            let mask = mask.split("=").nth(1).expect("Failed to parse mask");
            let instr = Instr {
                name: name.to_string(),
                mask_value: parse_quote!(#mask),
                match_value: parse_quote!(#march),
            };

            let value = u32::from_str_radix(march.trim_start_matches("0x"), 16)
                .expect("Failed to parse match value");

            match (value & 255) as u8 {
                0b1100011 => self.b.push(instr),
                0b1100111 => self.i.push(instr),
                0b1101111 => self.j.push(instr),
                0b0110011 => self.r.push(instr),
                0b0100011 => self.s.push(instr),
                0b0010111 => self.u.push(instr),
                _ => {}
            }
        }

        Ok(())
    }

    /// Download the riscv-opcodes repository
    fn download_opcodes(&self) -> Result<()> {
        let repo = self.root.join("riscv-opcodes");
        if repo.exists() {
            return Ok(());
        }

        Command::new("git")
            .args(["clone", RISCV_OPCODES_REPO, "--depth", "1"])
            .current_dir(&self.root)
            .status()
            .expect("Failed to download riscv/riscv-opcodes");

        Command::new("./parse.py")
            .args(PARSE_ARGS)
            .current_dir(repo)
            .status()
            .expect("Failed to build riscv/riscv-opcodes");

        Ok(())
    }
}

/// An RISC-V instruction
struct Instr {
    name: String,
    /// The mask value of the instruction
    mask_value: Expr,
    /// The match value of the instruction
    match_value: Expr,
}
