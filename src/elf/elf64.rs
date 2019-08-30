type Elf64Half = u16;
type Elf64Word = u32;
//type Elf64SWord = i32;
type Elf64Xword = u64;
//type Elf64Sxword = i64;
type Elf64Addr = u64;
type Elf64Off = u64;
type Elf64Section = u16;
type EIDENT = u128;
struct ELF {
    ehdr: Ehdr,
    shdrs: Vec<Shdr>,
    phdrs: Option<Vec<Phdr>>,
    symtab: Vec<Sym>,
    strtab: Vec<u8>,
}

impl ELF {
    fn new(binary: Vec<u8>) -> ELF {
        let ehdr: Ehdr = Ehdr::new_unsafe(binary[0..64].to_vec());
        let shdrs: Vec<Shdr> = ELF::build_shdrs(&ehdr, binary[ehdr.e_shoff as usize..].to_vec());
        let phdrs: Vec<Phdr> = ELF::build_phdrs(&ehdr, binary[ehdr.e_phoff as usize..].to_vec());
        let (symtab_vec, sym_number) = ELF::get_section(&ehdr, &shdrs, binary.to_vec(), ".symtab");
        let (strtab, _) = ELF::get_section(&ehdr, &shdrs, binary, ".strtab");
        let symtab = ELF::build_symtab(symtab_vec, sym_number);
        eprintln!("length -> {}", symtab.len());
        ELF {
            ehdr: ehdr,
            shdrs: shdrs,
            phdrs: if 0 < phdrs.len() { Some(phdrs) } else { None },
            symtab: symtab,
            strtab: strtab,
        }
    }
    fn get_section(
        ehdr: &Ehdr,
        shdrs: &Vec<Shdr>,
        binary: Vec<u8>,
        secname: &str,
    ) -> (Vec<u8>, u64) {
        let mut section = Vec::new();
        let mut sh_entsize = 0;
        let offset = shdrs[ehdr.e_shstrndx as usize].sh_offset as usize;
        let shstrtab =
            binary[offset..offset + shdrs[ehdr.e_shstrndx as usize].sh_size as usize].to_vec();
        for shdr in shdrs.iter() {
            if shdr.sh_offset == 0 {
                continue;
            }
            if get_string_with_null_terminated(shstrtab[shdr.sh_name as usize..].to_vec())
                == secname
            {
                section = binary[shdr.sh_offset as usize..(shdr.sh_offset + shdr.sh_size) as usize]
                    .to_vec();
                sh_entsize = shdr.sh_entsize;
            }
        }
        (section, sh_entsize)
    }
    fn build_shdrs(ehdr: &Ehdr, binary: Vec<u8>) -> Vec<Shdr> {
        let mut shdrs: Vec<Shdr> = Vec::new();
        for i in 0..ehdr.e_shnum {
            shdrs.push(Shdr::new_unsafe(
                binary[(i * ehdr.e_shentsize) as usize..].to_vec(),
            ));
        }
        shdrs
    }
    fn build_phdrs(ehdr: &Ehdr, binary: Vec<u8>) -> Vec<Phdr> {
        let mut phdrs: Vec<Phdr> = Vec::new();
        for i in 0..ehdr.e_phnum {
            phdrs.push(Phdr::new_unsafe(
                binary[(i * ehdr.e_phentsize) as usize..].to_vec(),
            ));
        }
        phdrs
    }
    fn build_symtab(v: Vec<u8>, entsize: u64) -> Vec<Sym> {
        let mut symtab: Vec<Sym> = Vec::new();
        for i in 0..entsize {
            symtab.push(Sym::new_unsafe(v[(i * Sym::size()) as usize..].to_vec()));
        }
        symtab
    }
    fn dump(&self) {
        self.ehdr.dump();
        for shdr in self.shdrs.iter() {
            shdr.dump();
        }
        if let Some(phdr_vec) = &self.phdrs {
            for phdr in phdr_vec.iter() {
                phdr.dump();
            }
        }
        for sym in &self.symtab {
            sym.dump(self.strtab.to_vec());
        }
    }
}
#[repr(C)]
struct Ehdr {
    e_ident: EIDENT,
    e_type: Elf64Half,
    e_machine: Elf64Half,
    e_version: Elf64Word,
    e_entry: Elf64Addr,
    e_phoff: Elf64Off,
    e_shoff: Elf64Off,
    e_flags: Elf64Word,
    e_ehsize: Elf64Half,
    e_phentsize: Elf64Half,
    e_phnum: Elf64Half,
    e_shentsize: Elf64Half,
    e_shnum: Elf64Half,
    e_shstrndx: Elf64Half,
}
impl Ehdr {
    fn new_unsafe(binary: Vec<u8>) -> Ehdr {
        unsafe { std::ptr::read(binary.as_ptr() as *const Ehdr) }
    }
    fn dump(&self) {
        eprintln!("Magicnumber(Little Endian) -> {:x}", self.e_ident);
        eprintln!("Type -> {}", self.e_type);
        eprintln!("Machine -> 0x{:x}", self.e_machine);
        eprintln!("Version -> {}", self.e_version);
        eprintln!("Entrypoint -> 0x{:x}", self.e_entry);
        eprintln!("Program Header Table Offset -> 0x{:x}", self.e_phoff);
        eprintln!("Section Header Table Offset -> 0x{:x}", self.e_shoff);
        eprintln!("Flags -> {:b}", self.e_flags);
        eprintln!("ELF-Header Size -> {}(bytes)", self.e_ehsize);
        eprintln!("Program-Header Size -> {}(bytes)", self.e_phentsize);
        eprintln!("Program-Header Number -> {}", self.e_phnum);
        eprintln!("Section-Header Size -> {}(bytes)", self.e_shentsize);
        eprintln!("Section-Header Number -> {}", self.e_shnum);
        eprintln!(".shstrtab Index -> {}", self.e_shstrndx);
    }
}

#[repr(C)]
struct Shdr {
    sh_name: Elf64Word,
    sh_type: Elf64Word,
    sh_flags: Elf64Xword,
    sh_addr: Elf64Addr,
    sh_offset: Elf64Off,
    sh_size: Elf64Xword,
    sh_link: Elf64Word,
    sh_info: Elf64Word,
    sh_addralign: Elf64Xword,
    sh_entsize: Elf64Xword,
}

impl Shdr {
    fn new_unsafe(binary: Vec<u8>) -> Shdr {
        unsafe { std::ptr::read(binary.as_ptr() as *const Shdr) }
    }
    fn dump(&self) {
        eprintln!("sh_name(.shstrtab index) -> {}", self.sh_name);
        eprintln!("Type -> {}", self.sh_type);
        eprintln!("Flags -> {:b}", self.sh_flags);
        eprintln!("Address -> 0x{:x}", self.sh_addr);
        eprintln!("Offset -> 0x{:x}", self.sh_offset);
        eprintln!("Size -> {}(bytes)", self.sh_size);
        eprintln!("Link -> {}", self.sh_link);
        eprintln!("Info -> {}", self.sh_info);
        eprintln!("Address-Alignment -> {}", self.sh_addralign);
        eprintln!("Entrysize -> {}", self.sh_entsize);
    }
}
#[repr(C)]
struct Phdr {
    p_type: Elf64Word,
    p_flags: Elf64Word,
    p_offset: Elf64Off,
    p_vaddr: Elf64Addr,
    p_paddr: Elf64Addr,
    p_filesz: Elf64Word,
    p_memsz: Elf64Xword,
    p_align: Elf64Xword,
}

impl Phdr {
    fn new_unsafe(binary: Vec<u8>) -> Phdr {
        unsafe { std::ptr::read(binary.as_ptr() as *const Phdr) }
    }
    fn dump(&self) {
        eprintln!("Type -> {}", self.p_type);
        eprintln!("flags -> {:b}", self.p_flags);
        eprintln!("offset -> 0x{:x}", self.p_offset);
        eprintln!("vaddr -> 0x{:x}", self.p_vaddr);
        eprintln!("paddr -> 0x{:x}", self.p_paddr);
        eprintln!("filesz -> {}(bytes)", self.p_filesz);
        eprintln!("memsz -> {}(bytes)", self.p_memsz);
        eprintln!("align -> {}", self.p_align);
    }
}
#[repr(C)]
struct Sym {
    st_name: Elf64Word,
    st_info: u8,
    st_other: u8,
    st_shndx: Elf64Section,
    st_value: Elf64Addr,
    st_size: Elf64Xword,
}

impl Sym {
    fn new_unsafe(binary: Vec<u8>) -> Sym {
        unsafe { std::ptr::read(binary.as_ptr() as *const Sym) }
    }
    fn size() -> u64 {
        24
    }
    fn dump(&self, strtab: Vec<u8>) {
        eprintln!(
            "name -> {}",
            get_string_with_null_terminated(strtab[self.st_name as usize..].to_vec())
        );
        eprintln!("info -> {}", self.st_info);
        eprintln!("other -> {}", self.st_other);
        eprintln!("shndx -> {}", self.st_shndx);
        eprintln!("value -> 0x{:x}", self.st_value);
        eprintln!("size -> {}", self.st_size);
    }
}

fn get_string_with_null_terminated(binary: Vec<u8>) -> String {
    let st = String::from_utf8(
        binary
            .into_iter()
            .take_while(|c| c != &0x00)
            .collect::<Vec<u8>>(),
    );
    st.unwrap()
}
pub fn dump_elf_in_detail(binary: Vec<u8>) {
    let elf_file: ELF = ELF::new(binary);
    elf_file.dump();
}
