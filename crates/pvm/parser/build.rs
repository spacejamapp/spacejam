//! Build script for the RISC-V parser

use anyhow::Result;
use heck::ToUpperCamelCase;
use proc_macro2::Span;
use quote::ToTokens;
use std::{env, fs, path::PathBuf, process::Command};
use syn::{parse_quote, ExprMatch, Ident, ItemEnum};

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
struct BuildContext {
    root: PathBuf,
    item_enum: ItemEnum,
    expr_match: ExprMatch,
}

impl BuildContext {
    fn new() -> Result<Self> {
        let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
        Ok(Self {
            root,
            item_enum: parse_quote!(
                /// RISC-V instruction
                #[derive(Debug, PartialEq, Eq, Clone, Copy)]
                pub enum Instruction {}
            ),
            expr_match: parse_quote!(match bits {}),
        })
    }

    fn write_instr_rs(&self) -> Result<()> {
        let mut instr_rs = String::new();
        instr_rs.push_str("use crate::format::{RType, IType, SType, BType, UType, JType};\n");

        let item_enum = self.item_enum.clone();
        let expr_match = self.expr_match.clone();
        instr_rs.push_str(
            quote::quote! {
                #item_enum

                impl TryFrom<u32> for Instruction {
                    type Error = anyhow::Error;

                    fn try_from(bits: u32) -> anyhow::Result<Self> {
                        Ok(#expr_match)
                    }
                }
            }
            .to_token_stream()
            .to_string()
            .as_str(),
        );

        fs::write(
            PathBuf::from(env::var("OUT_DIR")?).join("instr.rs"),
            instr_rs,
        )?;
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

            let (name, march, mask) = {
                let mut matches = march.split_ascii_whitespace();
                let name = matches
                    .nth(1)
                    .expect("Failed to parse name")
                    .trim_start_matches("MATCH_")
                    .trim_end_matches(':');

                let march = matches
                    .last()
                    .expect("Failed to parse match")
                    .trim_start_matches("0x")
                    .trim_end_matches(';')
                    .trim();

                let mask = mask
                    .split("=")
                    .nth(1)
                    .expect("Failed to parse mask")
                    .trim()
                    .trim_start_matches("0x")
                    .trim_end_matches(';');

                (name, march, mask)
            };

            let match_value = u32::from_str_radix(march, 16).expect("Failed to parse match value");
            let mask_value = u32::from_str_radix(mask, 16).expect("Failed to parse mask value");

            let format = Ident::new(
                match (match_value & 255) as u8 {
                    0b1100011 => "BType",
                    0b1100111 => "IType",
                    0b1101111 => "JType",
                    0b0110011 => "RType",
                    0b0100011 => "SType",
                    0b0010111 => "UType",
                    _ => continue,
                },
                Span::call_site(),
            );

            let variant_name = Ident::new(&name.to_upper_camel_case(), Span::call_site());
            self.item_enum.variants.push(parse_quote! {
                #[doc = concat!("RISC-V `", #name, "` instruction")]
                #variant_name(#format)
            });

            self.expr_match.arms.push(parse_quote! {
                instr if instr ^ #match_value & #mask_value == 0 => Self::#variant_name(#format::from(instr.to_le_bytes()))
            });
        }

        self.expr_match.arms.push(parse_quote! {
            _ => anyhow::bail!("Invalid instruction")
        });
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
