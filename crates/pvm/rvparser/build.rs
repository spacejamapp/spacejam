//! Build script for the RISC-V parser

use anyhow::Result;
use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::Span;
use quote::ToTokens;
use std::{env, fs, path::PathBuf, process::Command};
use syn::{parse_quote, Arm, ExprMatch, Ident, ItemEnum, ItemFn, ItemTrait};

const RISCV_OPCODES_REPO: &str = "https://github.com/riscv/riscv-opcodes.git";
const PARSE_ARGS: [&str; 3] = ["-rust", "rv_i", "rv_m"];

fn main() -> Result<()> {
    let mut ctx = BuildContext::new()?;
    ctx.download_opcodes()?;
    ctx.read_instructions()?;
    ctx.write_instr_rs()?;
    ctx.write_visitor_rs()?;
    Ok(())
}

/// Opcodes build context
struct BuildContext {
    root: PathBuf,
    item_enum_instr: ItemEnum,
    item_trait_visitor: ItemTrait,
    expr_match_encode: ExprMatch,
    expr_match_parse: ExprMatch,
    expr_match_visit: ExprMatch,
    expr_match_visit_u32: ExprMatch,
    parser_tests: Vec<ItemFn>,
}

impl BuildContext {
    fn new() -> Result<Self> {
        let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
        Ok(Self {
            root,
            item_enum_instr: parse_quote!(
                /// RISC-V instruction
                #[derive(Debug, PartialEq, Eq, Clone, Copy)]
                pub enum Instruction {}
            ),
            item_trait_visitor: parse_quote!(
                /// RISC-V instruction visitor
                pub trait Visitor {
                    /// Visit an instruction in bytes
                    fn visit_bytes(&mut self, bytes: [u8; 4]) -> anyhow::Result<()> {
                        Self::visit(self, u32::from_le_bytes(bytes))
                    }
                }
            ),
            expr_match_encode: parse_quote!(match instr {}),
            expr_match_parse: parse_quote!(match bits {}),
            expr_match_visit: parse_quote!(match instr {}),
            expr_match_visit_u32: parse_quote!(match instr {}),
            parser_tests: vec![],
        })
    }

    fn write_instr_rs(&self) -> Result<()> {
        let item_enum_instr = self.item_enum_instr.clone();
        let expr_match_parse = self.expr_match_parse.clone();
        let expr_match_encode = self.expr_match_encode.clone();
        let parser_tests = self.parser_tests.clone();
        let instr_rs = quote::quote! {
            #item_enum_instr

            impl TryFrom<u32> for Instruction {
                type Error = anyhow::Error;

                fn try_from(bits: u32) -> anyhow::Result<Self> {
                    Ok(#expr_match_parse)
                }
            }

            impl From<Instruction> for u32 {
                fn from(instr: Instruction) -> Self {
                    #expr_match_encode
                }
            }

            #(#parser_tests)*
        };

        fs::write(
            PathBuf::from(env::var("OUT_DIR")?).join("instr.rs"),
            instr_rs.to_token_stream().to_string(),
        )?;
        Ok(())
    }

    fn write_visitor_rs(&self) -> Result<()> {
        let mut item_trait_visitor = self.item_trait_visitor.clone();
        let expr_match_visit_u32 = self.expr_match_visit_u32.clone();
        let expr_match_visit = self.expr_match_visit.clone();
        item_trait_visitor.items.append(&mut vec![
            parse_quote! {
                /// Visit an instruction in u32
                fn visit(&mut self, instr: u32) -> anyhow::Result<()> {
                    #expr_match_visit_u32
                }
            },
            parse_quote! {
                /// Visit an instruction
                ///
                /// NOTE: You you need to parse instruction from bytes or u32 first,
                /// please use [Self::visit] directly.
                fn visit_instr(&mut self, instr: Instruction) -> anyhow::Result<()> {
                    #expr_match_visit
                }
            },
        ]);

        fs::write(
            PathBuf::from(env::var("OUT_DIR")?).join("visitor.rs"),
            item_trait_visitor.to_token_stream().to_string(),
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

            let name_snake = Ident::new(&name.to_snake_case(), Span::call_site());
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

            // prepare shared tokens
            let variant_name = Ident::new(&name.to_upper_camel_case(), Span::call_site());
            let mut marm: Arm = parse_quote!(instr if instr ^ #match_value & #mask_value == 0 => Self::#variant_name(#format::from(instr)));
            let mut iarm: Arm = parse_quote!(Instruction::#variant_name(instr) => instr.into());

            // parser codegen
            self.item_enum_instr.variants.push(parse_quote! {
                #[doc = concat!("RISC-V `", #name, "` instruction")]
                #variant_name(#format)
            });

            self.expr_match_encode.arms.push(iarm.clone());
            self.expr_match_parse.arms.push(marm.clone());
            self.parser_tests.push(parse_quote! {
                #[test]
                fn #name_snake() {
                    let instr = Instruction::try_from(#match_value).expect(format!("Failed to parse instruction: {}", #name).as_str());
                    assert_eq!(instr, Instruction::#variant_name(#format::from(#match_value)));
                    assert_eq!(u32::from(instr), #match_value);
                }
            });

            // visitor codegen
            self.item_trait_visitor.items.push(parse_quote! {
                #[doc = concat!("Visit `", #name, "` instruction")]
                fn #name_snake(&mut self, instr: #format) -> anyhow::Result<()>;
            });

            iarm.body = parse_quote!(Self::#name_snake(self, instr));
            self.expr_match_visit.arms.push(iarm);

            marm.body = parse_quote!(Self::#name_snake(self, instr.into()));
            self.expr_match_visit_u32.arms.push(marm);
        }

        let invalid: Arm = parse_quote!(_ => anyhow::bail!("Invalid instruction"));
        self.expr_match_parse.arms.push(invalid.clone());
        self.expr_match_visit_u32.arms.push(invalid.clone());
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
