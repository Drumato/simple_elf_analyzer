mod elf;
fn main() -> Result<(), Box<std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: ./main <binary-file>");
        std::process::exit(1);
    }
    let content: Vec<u8> = read_file("a.out".to_string())?;
    elf::elf64::dump_elf_in_detail(content);
    Ok(())
}
fn read_file(filename: String) -> Result<Vec<u8>, Box<std::error::Error>> {
    use std::fs::File;
    use std::io::Read;
    let mut file: File = File::open(filename)?;
    let mut content: Vec<u8> = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(content)
}
