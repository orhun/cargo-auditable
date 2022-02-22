//! Shamelessly copied from rustc codebase:
//! https://github.com/rust-lang/rust/blob/3b186511f62b0ce20e72ede0e8e13f8787155f02/compiler/rustc_codegen_ssa/src/back/metadata.rs#L260-L298

mod format_guess;

use object::write::{self, StandardSegment, Symbol, SymbolSection};
use object::{
    elf, Architecture, BinaryFormat, Endianness, FileFlags, Object, ObjectSection, SectionFlags,
    SectionKind, SymbolFlags, SymbolKind, SymbolScope,
};

use format_guess::{FormatDescription, guess_format};

pub fn create_metadata_file(
    format: &FormatDescription,
    compressed: &[u8],
    symbol_name: &str,
) -> Vec<u8> {
    let mut file = create_object_file(format);
    let section = file.add_section(
        file.segment_name(StandardSegment::Data).to_vec(),
        b".rustc".to_vec(),
        SectionKind::ReadOnlyData,
    );
    match file.format() {
        BinaryFormat::Elf => {
            // Explicitly set no flags to avoid SHF_ALLOC default for data section.
            file.section_mut(section).flags = SectionFlags::Elf { sh_flags: 0 };
        }
        _ => {}
    };
    let offset = file.append_section_data(section, &compressed, 1);

    // For MachO and probably PE this is necessary to prevent the linker from throwing away the
    // .rustc section. For ELF this isn't necessary, but it also doesn't harm.
    file.add_symbol(Symbol {
        name: symbol_name.as_bytes().to_vec(),
        value: offset,
        size: compressed.len() as u64,
        kind: SymbolKind::Data,
        scope: SymbolScope::Dynamic,
        weak: false,
        section: SymbolSection::Section(section),
        flags: SymbolFlags::None,
    });

    file.write().unwrap()
}

fn create_object_file(f: &FormatDescription) -> write::Object<'static> {
    // The equivalent function inside rustc contains spooky special-casing for MIPS and RISC-V:
    // https://github.com/rust-lang/rust/blob/03a8cc7df1d65554a4d40825b0490c93ac0f0236/compiler/rustc_codegen_ssa/src/back/metadata.rs#L133-L165
    // I am ignoring that in the prototype for now.
    // To get this into Cargo, presumably we will need a way to share that code between rustc and Cargo.
    // -- Shnatsel
    write::Object::new(f.format, f.architecture, f.endian)
}

fn main() {
    let target_triple = std::env::args().nth(1).unwrap_or_else(|| usage() );
    let format = guess_format(&target_triple);
    

}

fn usage() -> ! {
    eprintln!("Usage: auditable-inject target-triple /path/to/data_to_inject /path/to/output_file");
    std::process::exit(1);
}